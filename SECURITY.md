# Política de seguridad — Arqueo (antes ZK-SSL)

## Léelo antes que el resto

Este texto describe **intenciones de diseño y limitaciones medidas**, no
garantías auditadas:

- **Nada de este proyecto ha sido auditado por terceros.** Ninguna cantidad de
  pruebas propias sustituye una auditoría externa. No la hay.
- Las propiedades de seguridad listadas son **objetivos del diseño**, validados
  —cuando lo están— por los tests del propio autor.
- Los problemas de §3 son **hallazgos internos**. La mayoría están medidos y
  con referencia a `AUDITORIA.md`; **re-verifícalos contra el código actual**
  antes de confiar en ellos o darlos por resueltos.
- La fuente autorizada del comportamiento real es el código. **Si este
  documento y el código discrepan, el código gana y este documento está mal.**

⚠️ Ese último punto no es retórica. Este proyecto ha registrado **cuatro veces**
una garantía enunciada en prosa cuya condición nadie verificó
(`AUDITORIA.md` §95.2, §98.4). Si algo aquí suena más seguro de lo que puedes
verificar tú leyendo el repositorio, trátalo como afirmación pendiente.

---

## Estado del proyecto

- **Prototipo de investigación**, no un producto. No maneja dinero real y no
  debe manejarlo en su estado actual.
- **Nodo único operado por una sola parte.** Pese a la palabra «Sovereign» del nombre anterior,
  **no es descentralizado**: el operador ve el estado, ordena las operaciones y
  puede censurar. Es una característica declarada, no un descuido.
- **Su tesis no es la evasión**: privacidad frente a terceros con supervisión
  demostrable —revelación selectiva del titular, límites de emisión en
  circuito—.
- **Su aportación medible** es la comparación empírica de cinco sistemas de
  prueba sobre el mismo circuito.

---

## 1. Qué pretende garantizar el diseño

- **Caducidad del pendiente con doble cerrojo** (`AUDITORIA.md` §178-§181):
  pasada la `T` declarada, el emisor recupera un envío no cobrado — y SOLO
  el emisor: el destino lo fijan los registros de la capa (no la prueba) y
  la subida de crédito exige el salt derivado de SU clave (§117). Testigos:
  `el_ladron_con_aviso_no_puede_reembolsarse`,
  `la_carrera_post_t_el_primero_gana`,
  `las_dos_vias_de_caducidad_no_se_cruzan`. Los pendientes de emisión
  caducan destruyendo (el suministro baja lo que subió).

**Propiedades objetivo.** Su corrección depende de que las restricciones de
circuito sean completas, y eso **no está formalmente especificado ni auditado**
(§3.1).

- **No creación de dinero**: una transición válida conserva el valor total.
- **Autoridad de gasto**: solo quien controla la clave puede gastar.
- **No reescritura silenciosa del historial**: el registro encadenado impide
  reescrituras **detectables por quien haya observado una cabeza anterior**.
  ⚠️ Garantía condicional: alguien distinto del operador tiene que observar
  sus cabezas. Desde el §245 existe el testigo que las anota (§2), pero uno
  que opere el propio operador no prueba nada.
- **Privacidad del contenido frente a terceros**: el compromiso de hoja oculta
  el saldo desde la entrada 50 (§3.2); lo que las pruebas ocultan desde el
  §538, y lo que no se promete, lo dice la sección 3.bis. El operador lo ve
  todo (§2), y nada de esto está auditado.

⚠️ **Corregido en el §698**: hasta entonces la tercera decía «hoy nadie fuera del operador
observa cabezas» y la cuarta, «medido que no se cumple», contra el §2 y el §3.2 de este mismo
documento.

Ninguna debe citarse fuera de este repositorio como «garantizada». La
formulación honesta es: *«el diseño pretende X; no está auditado»*.

⚠️ **Y dos de estas propiedades ya fallaron en implementación** mientras el
diseño las garantizaba: hasta el 31-07-2026 la capa no verificaba las pruebas
de la vía de pago (`AUDITORIA.md` §73) y el compromiso del pendiente no estaba
atado al importe (§74). Ambas corregidas y medidas.

## 2. Qué NO protege

- **Sin demostración formal de los layouts (sub-restringimiento).** La
  suite prueba PUNTOS (testigos válidos verifican; los corruptos que se
  nos ocurrieron rebotan), no el universal «ningún testigo-fantasma
  satisface las restricciones».

  **Lo que SÍ existe hoy (§195-§196), y lo que aún no.** El núcleo de
  pagos tiene **ESPEC ejecutable**: un intérprete fino reproduce byte a
  byte la salida patrón-oro del circuito Rust —mutantes incluidos— y una
  compuerta cuenta **cada celda de la traza con dueño declarado**
  (`circuit_send`: 23 clases · 1334 celdas-clase · **0 sin dueño**;
  `circuit_claim`: 21 · 1197 · 0). Eso cierra una pregunta concreta —«¿hay
  celdas que nadie restringe?»— y **no cierra** la que importa: que las
  restricciones existentes sean *suficientes*. FV-1 (censo) está HECHO;
  FV-2 (spike SMT acotado) y FV-3 (Lean/K) siguen siendo horizonte
  declarado. Ver `doc/VERIFICACION_FORMAL.md` y `doc/fv/`.

  ⚠️ **Y esta clase no es teórica: es la que rompe sistemas en
  producción.** En junio de 2026 se divulgó un fallo de
  **sub-restringimiento en el gadget de multiplicación elíptica de
  halo2** que afectaba al pool Orchard de Zcash —el sistema de dinero ZK
  más maduro que existe— **expuesto durante cuatro años** y detectado por
  una auditoría asistida por IA, no por su suite. La respuesta de aquel
  ecosistema fue exactamente el camino que aquí está declarado: nuevo
  pool auditado y **esfuerzo de verificación formal** para probar que esa
  clase no puede repetirse. Si un proyecto con auditorías, años de
  producción y un equipo dedicado tardó cuatro años en verla, **este
  repositorio —sin auditoría externa— no puede llamar sólidos a sus
  circuitos**, y no lo hace.

- **La carrera post-T es del operador.** Tras la `T` de caducidad, cobro y
  reembolso compiten y el orden dentro del lote lo decide quien ordena —
  es el residuo de orden general (§121), no una superficie nueva; la
  carrera está declarada y probada en ambos órdenes. Los pendientes
  ANTERIORES a la caducidad no tienen meta y son inmunes: solo cobro,
  para siempre.
- **Inmovilización, no robo, en la vía retirada** (`AUDITORIA.md` §177):
  la única deuda demostrable del árbol de nullifiers heredado es que el
  operador puede inmovilizar; nunca gastar ni redirigir.

- **El operador ve el estado.** Es el mayor límite de privacidad y está
  asumido.
- **El operador puede censurar y ordenar.** Desde el RFC-0010, censurar DESPUÉS del
  recibo deja evidencia portable; ANTES del recibo no deja ninguna, y eso ya no es
  sólo declarado: lo mide el §601 (`tools/banco_mentiroso_censura.sh`, el residuo D-H).
  La de después también está medida: un nodo que responde con recibo y deshace la
  operación queda nombrado al expirar la ventana (§602,
  `tools/banco_mentiroso_sin_resolver.sh --largo`). ⚠️ §815: y hasta el §815 quedaba nombrado
  igual un nodo HONRADO que se caía entre anotar el recibo y aplicar la operación, y uno al
  que un tercero sin cuenta mandaba una prueba basura: los dos, en la 3.15.
- **La custodia del registro está en manos del operador.** Sin observadores
  externos de sus cabezas, podría presentar historias distintas a partes
  distintas. **Desde §241-§243 el nodo firma, emite y sirve sus cabezas, y
  existe un verificador independiente** (`zk-ssl-verify`); **desde el §245 hay
  testigo, con TOFU** — pero uno que opere el propio operador no prueba nada.

- ⚠️ **El ancla de la clave pública: TOFU desde §245, y nada antes.**

  Hasta §244 esto era un bloqueante sin salida. **El testigo lo resolvió a
  medias, y por una vía que estaba a la vista**: un testigo que anota la
  clave que ve **la primera vez** y **se detiene si cambia** hace
  *trust-on-first-use* — el modelo de SSH.

  **Lo que TOFU sí da**: desde el primer encuentro, el operador **no puede
  cambiar de clave sin que un tercero lo vea**. Y rotar es exactamente cómo
  escaparía de una vista dividida, así que el testigo **se detiene ante las
  dos cosas**.

  ⚠️⚠️ **Lo que TOFU NO da: el primer encuentro.** Si el operador ya
  mentía cuando el testigo arrancó, **TOFU fija la mentira**. Es una
  limitación **del modelo**, no de la implementación, y no se cierra con
  más código: exige un ancla **anterior** —una huella publicada, una
  autoridad, una contraparte—. Lo que TOFU aporta es **acotar la ventana a
  un instante** en vez de dejarla abierta para siempre.

  ⚠️ Y **un testigo que opera el propio operador no prueba nada**: es
  circular. Lo que el proyecto puede dar es la implementación de
  referencia — **quita la excusa de que no hay cómo**, no la desconfianza.

  ⚠️ **Y el ancla anterior no está sin decidir: NO HAY CLAVE QUE ANCLAR**
  (§246). El operador **no tiene ninguna**; `--clave` existe *«para
  ejercitar el mecanismo, no como forma de operar»*, y así está escrito en
  su propia documentación.

  Anclar hoy sería **anclar una clave de prueba**. La decisión del ancla
  **viene con la decisión de despliegue**, no antes — y eso la reformula:
  no es «falta elegir entre cuatro opciones», es «falta la clave».

  Hoy un tercero verifica que la firma cuadra con **la clave que el mismo
  nodo le dio**. Eso es circular: un operador puede cambiar de clave entre
  dos consultas y **ambas respuestas verifican**.

  ⚠️ Aunque la clave privada viviera en un HSM con tres custodios, **el
  testigo seguiría sin poder afirmar de quién es la firma**. El eslabón que
  falta no está en el operador: está en **qué ancla usa el tercero**.

  Opciones enumeradas, **ninguna elegida**: una huella publicada fuera del
  nodo · un registro de transparencia · una contraparte que la ancle · una
  autoridad de certificación. **Se elige según quién vaya a usar el ancla, y
  no hay nadie** — decidirlo ahora sería fijar la forma antes del dato, que
  es lo que §242 evitó con el histórico.

  Es lo que Certificate Transparency resuelve publicando las claves de log
  **fuera del log**.

- **Custodia de la clave privada: declarada, no comprobada** (§244). El
  operador afirma un modelo con `--custodia`, y el nodo lo sirve en
  `zkssl_signedEpochHead`. ⚠️ **Solo `fichero` se comprueba**; el resto son
  afirmaciones suyas. El valor no está en que sean ciertas, **sino en que
  mentir en ellas es oponible**.
- ⚠️ **El índice de la clave de firma: un solo proceso por contador, y lo que
  eso no cubre** (§709). XMSS tiene estado: cada firma gasta una hoja, y dos
  mensajes firmados con la misma hoja comprometen la clave. El guardián del
  índice persiste su contador con `fsync` antes de cada firma y, desde el §709,
  toma además un cerrojo exclusivo sobre el fichero: un segundo proceso sobre el
  mismo contador —otro nodo, otro cofirmante del testigo— **no arranca**, y el
  error dice qué proceso lo tiene, si el sistema lo enseña. Al firmar,
  `--indice-firma` y `--contador-recepcion` del nodo y `--indice-cofirma` del
  testigo son obligatorias y van con su ruta absoluta. Hasta el §709 las dos del
  nodo tenían un valor por defecto relativo al directorio de trabajo, y bastaba
  lanzar dos nodos desde el mismo sitio para que compartieran contador; el
  cofirmante del testigo no excluía a un segundo sobre el suyo.
  **Lo que el cerrojo NO cubre**: un sistema de ficheros de red, donde el
  cerrojo puede no cruzar de una máquina a otra; otra máquina, o una copia del
  contador; **la misma semilla con otro contador**, porque dos ficheros son dos
  cerrojos; y un proceso que abra el fichero sin pedir el cerrojo, que en unix
  es de aviso y no obligatorio. Tampoco hace nada frente a un contador que
  vuelve atrás o se pierde: eso lo declara `doc/CONFIANZA_RESIDUAL.md`.
  ⚠️ §860: por eso una réplica o un relevo no lleva el contador ni la semilla a
  otra máquina que pueda firmar; con el estado de la clave no fiable, el
  relevo es la D-G del RFC-0015, PROPUESTO; y lo que el nodo no impide —la
  misma clave en dos sitios, con dos contadores— es la entrada 124 del
  `BACKLOG.md`. La alta disponibilidad del operador ni la construye el
  proyecto ni la prescribe el estándar (la 17, cerrada).
- ⚠️ **El operador puede cambiar el verificador, y hoy eso es invisible.**
  Es **el poder mayor de todos** y no estaba en esta lista: quien puede
  actualizar el verificador **cambia qué es una transición válida** —más
  poderoso que cualquier operación, porque **redefine las reglas bajo las que
  todas las demás se juzgan**—. Un operador que lo sustituye puede aceptar
  como válido lo que las reglas publicadas rechazarían, **sin dejar rastro en
  el estado**. No hay noción de «reglas vigentes» hoy —`OpKind` dice qué
  circuito usar, no qué versión estaba activa—, así que el cambio **no queda
  registrado ni es comprobable a posteriori**. El cierre diseñado
  —`hash_verificador_vigente` en la cabeza atestiguada, que vuelve pública
  toda actualización— está en `doc/CONFIANZA_RESIDUAL.md` §2.2 y es la
  entrada 54; **requiere primero dar al sistema esa noción**.
  ⚠️ **CORRECCIÓN (§321, 2026-08-19): la condición de la línea
  anterior está corregida desde el §246.** «Dar al sistema esa
  noción» es casi circular —`hash_verificador_vigente` ES ese
  mecanismo—. La razón real por la que no puede rellenarse hoy: **el AIR
  es CÓDIGO, no datos**. Lo único hasheable en ejecución son las
  `ProofOptions`, y **un operador puede cambiar el AIR dejándolas
  idénticas**: el campo no sería vacío, sería **ciego** — y un campo
  ciego pasa desapercibido mintiendo justo sobre lo que existe para detectar.
  Depende de la **entrada 55** (el AIR como datos) o de **compilación
  reproducible**; hashear el fuente al compilar no vale, porque no prueba que
  el binario se construyera de ese fuente. La línea anterior se conserva y
  se cita en vez de borrarla (§247).
- **No hay recuperación si el nodo desaparece.**
- **Metadatos**: qué posiciones cambian y cuándo siguen siendo observables.
  Medido campo a campo en §231: un envío revela **emisor, importe y
  `notice.position`**; un cobro revela **receptor, importe y la misma
  posición**. Entre el §523 y el §538 la prueba del envío publicaba además
  **la identidad del receptor** (medido en la sesión 163, §523), así que
  quien procesara solo envíos veía la arista entera; desde el §538 la
  prueba la oculta (§3.bis). El nodo la ve siempre, por la prueba o no:
  recibe `receiverId` en `zkssl_sendMaterials`. Un **agregador** (§223)
  que solo vea envíos ya no la lee en la prueba, y lo que pueda deducir
  de una prueba lo acota el RFC-0009 (D-A, D-I), no este documento.
- **Solidez de circuitos y del sistema de prueba**: no verificada formalmente.
- ⚠️ **Rotar la clave exige dos custodios.** Un titular puede gastar sin
  permiso de nadie y **no puede mejorar su propia seguridad sin permiso de
  dos** (§98.4).
  ⚠️ §860: ya no del todo. La vista comprometida sola y la higiene sin sospecha
  no lo exigen: el traspaso, la R4 de `doc/ROTACION_DE_CUENTAS.md`, sin
  código, mientras quede plaza en el tope de cuentas (la 8); pero no retira ni
  la clave ni la identidad viejas, que siguen cobrando y abriendo la vista. El
  robo, la pérdida y cualquier sospecha sobre `sk` siguen exigiendo a los dos
  custodios —la recuperación— hasta que se construya la rotación
  precomprometida en la sal (la D-1 de `doc/ROTACION_DE_CUENTAS.md` §6, la 52
  del `BACKLOG.md`).
- ⚠️ **La gobernanza no rota: su compromiso y su pérdida son para siempre en
  su libro** (§860; la D-3 de `doc/ROTACION_DE_CUENTAS.md`). La raíz se fija al
  constituir el libro y al reabrirlo se compara. Una clave filtrada se queda
  en el conjunto, y el margen baja en una clave para siempre. Con dos
  filtradas, el umbral, quien las tenga rota los custodios a los suyos; la
  gobernanza legítima puede volver a rotarlos, pero lo que emitan o recuperen
  entretanto no se deshace. Si quedan menos de dos claves, los custodios no
  vuelven a rotar en ese libro y, agotado su cupo de intervenciones, no
  emiten, ni congelan, ni recuperan, salvo que el operador suba el cupo, que
  no protege ninguna autorización. La salida es un libro nuevo, y llevar a él
  a los titulares exige el reclamo del B18.2, que no existe.
  `PRINCIPIOS.md` §4.1.
- ⚠️ **Abrir una cuenta no se autoriza, y el tope de cuentas se agota para
  siempre** (§860; la 8 del `BACKLOG.md`, cerrada). `zkssl_openAccount` abre con
  `open_with_id`, sin prueba y sin límite de ritmo
  (`crates/zk-ssl-node/src/main.rs:3783-3796`), y quien llegue a él puede
  agotar el tope de cuentas (`max_accounts`, 1.000 por defecto en el nodo,
  `main.rs:248`). El tope es inmutable y una cuenta no se cierra, así que
  agotarlo deja el libro sin altas para siempre, y quita también el traspaso
  —la R4 de `doc/ROTACION_DE_CUENTAS.md` §4—, la única rotación sin custodios
  que existe hoy; y cada R4 legítima gasta una plaza que no vuelve, sin
  atacante. Es un ataque de vivacidad, y la vivacidad es hoy del operador, que
  decide quién llega al método: la falta de autenticación y de límite de tasa
  del nodo, y su `127.0.0.1` por defecto, los declara la sección 3.3; lo nuevo
  es que el tope no vuelve. Se decidió no exigir autorización de custodios
  para abrir: sería un segundo veto sobre el alta sin quitar el del operador.
  Y la prueba de posesión de la entrada 119 no lo cierra: prueba quién puede
  ocupar una identidad, no cuántas identidades frescas se abren.

### 2.bis Los dos residuos que quedan, y qué los elimina

Tras la entrada 50, las confianzas residuales de este diseño se reducen
a dos, y conviene nombrarlas mirando hacia delante:

1. **El orden y la completitud del historial.** El operador decide qué
   entra y en qué orden, y podría omitir. Mitigación diseñada: cabezas
   atestiguadas, recibos y **acuse** (§121,
   `doc/CONFIANZA_RESIDUAL.md`) — mentir pasa a dejar evidencia
   fail-stop. Eliminación: consenso/replicación o anclaje externo de
   raíces (formato y verificador construidos: RFC-0012,
   `doc/ANCLAJE_EXTERNO.md`; pendiente de despliegue en un medio, con su
   cadencia `M` por medir).

2. **El operador ve el estado.** La privacidad es frente a terceros que
   solo ven raíces y cabezas firmadas, no frente a quien mantiene el
   ledger; frente a quien ve una prueba, desde el §538 (§3.bis); el
   titular tiene vista autenticada (49-A) y el resto es asumido y
   documentado. Eliminación: arquitectura de operador ciego (B11) o
   federación.

Ninguna prueba ZK sustituye estas dos; lo que este proyecto exige es
que estén **escritas, medidas y con su ataque diseñado** en vez de
escondidas en la palabra «descentralizado».

### 2.ter El modelo de agregador — recomendación, no requisito

⚠️ **Ninguna de las dos mitades de esto existe.** Ni el turno ni el reparto
están construidos ni medidos: el cable los permite tal como está, y eso es
todo lo que se sabe. Esta sección fija una decisión de despliegue **antes**
de que alguien la tome por comodidad, no describe algo que funcione.

⚠️ Y es una **recomendación de mesa sin contraparte que la valide**. Si
mañana aparece un participante real, lo primero que preguntará es **quién
opera la mitad de cobros**, y esa pregunta no la responde este repositorio.

#### La decisión

**Turno explícito, y dos agregadores alternados** — uno de envíos, otro de
cobros.

El argumento no es de rendimiento: es que **el agregador único no necesita
el grafo para su trabajo**. Junta recibos y los manda en una petición. Ver
quién paga a quién no le aporta nada a esa función; es un efecto colateral
de la comodidad. **Conceder observación que no hace falta se defiende mal
después.**

Y el coste está medido: **cero en el nodo**. El turno hace falta de todas
formas porque la raíz lo exige (§230); el reparto viaja encima.

#### Lo que la propiedad da, y lo que NO

- **Cada agregador ve media arista**: emisor + importe, o receptor +
  importe, más `notice.position` (§231). ⚠️ **Eso no es cero: es
  información comercial.** Quién paga, cuánto y con qué frecuencia basta
  para muchos análisis.
- ⚠️ **Si la misma entidad opera ambas mitades, la propiedad se pierde
  entera y nadie lo nota desde fuera.** Se escribe como **condición**, no
  como supuesto: el sistema no la comprueba ni puede.
- **El nodo ve ambas mitades**, por definición. Esto es confidencialidad
  **entre participantes**, no frente al operador. Ya estaba dicho en §2.2;
  conviene que esta sección no se lea al revés.
- ⚠️ **Dos agregadores son dos puntos de censura, no medio cada uno.** El
  de envíos bloquea un pago **entero** sin tocar el cobro. Eso cae bajo el
  **recibo de admisión** (§121, `doc/CONFIANZA_RESIDUAL.md`) — y es la
  **cuarta** convergencia hacia esa pieza, que sigue sin construir.
  ⚠️ **Desde el RFC-0010 el recibo de recepción existe, y para lo
  agregado desde el §611.** Hasta entonces cubría `applySend` y
  `applyClaim` (D-E, §576), y el lote que manda un agregador por
  `zkssl_applyMany` no lo llevaba. El RFC-0014 (§608) propuso el del lote
  -y el de la prenda-, y desde su E3 (§611) el nodo lo emite: UNO por
  lote, sobre la huella de su composición, que el agregador reenvía a
  cada titular con la respuesta, y desde su E4 (§612) el sobre de
  completitud lo resuelve: aplicado, rechazado con prueba, por su forma
  -o RECHAZO SIN FUNDAMENTO, si la composición no la sostiene-, o
  declarado. El agregador que no reenvía queda fuera (D-C, D-H): lo elige
  el titular, y la vía directa sigue abierta; el agregador es una
  recomendación, no un requisito.

#### Lo que queda abierto

- **El coste del lote mixto frente a dos separados no está medido.** J.1
  midió que el mixto se admite; la recomendación asume que separar cuesta
  un turno, pero **cuánto cuesta ese turno no se sabe**. §230 acota el
  precio de **no** turnarse —el que pierde tira el 75 % de sus pruebas—,
  no el de turnarse.
- El mecanismo de turno —candado externo, cola, testigo rotatorio— no está
  elegido. Solo se afirma que **hace falta uno** y que **no es un
  consenso**: ver `spec/RPC.md`.

## 3. Problemas de seguridad identificados

> Hallazgos internos, **no auditados**. Casi todos están **medidos** con
> fecha y referencia; se listan porque ocultar debilidades conocidas sería
> lo contrario de la imagen fiel. Uno de ellos (§3.3) **cambió de forma**
> al aparecer la capa de red en §197: léelo entero aunque lo conocieras.

### 3.1 Ausencia de especificación formal del AIR — **prioridad más alta**

Un circuito sin restricciones completas puede admitir un testigo fraudulento
que pase la verificación. **Un fallo de solidez es dinero falso invisible.**

Mientras no exista una especificación formal de cada AIR —qué se restringe, de
qué grado, y qué explícitamente **no** se restringe— con tests de solidez
negativos, la propiedad de «no creación de dinero» descansa sobre una
suposición no comprobada.

⚠️ **Y no es hipotético en este proyecto**: §72 registra una restricción
escrita **sobre el carril equivocado** —bien formada, de grado correcto, y
atando lo que no era—. Ninguna herramienta la detectó.

**Estado: abierto.** Backlog 48 (B12.1).

### 3.2 El compromiso de hoja **ya ES ocultante** — MEDIDO y RESUELTO (entrada 50)

**Mundo viejo**: `native_leaf(identity, balance, nonce)` = `H(H(id,
saldo), nonce)`, **sin salt**. Y `path_for` entrega al cliente el hermano de nivel 0, que **es la hoja
de la cuenta vecina**.

**Medido el 31-07-2026: el saldo del vecino se recuperaba en 10,84 s.**

El coste es una **curva** sobre el rango de saldo que el atacante asuma:

| rango asumido | coste, un núcleo |
|---|---|
| 0–10.000 € | **2,4 min** |
| 0–1 M € | 4,1 h |
| 64 bits uniformes | 8,3 × 10⁷ años-núcleo — **que nunca lo son en dinero** |

Alcance: **una cuenta** por camino. Y los índices eran secuenciales, así
que **el vecino se elegía**. Ambas cosas, muertas — ver abajo y §157.

La solución que §99.3 descartaba —derivar en cada escritura— **no hizo
falta**: el salt se fija UNA vez al abrir (`derive_leaf_salt(sk)`, §117)
y se ALMACENA en el récord; quien escribe sin el secreto **lee**
`r.leaf_salt`, y recovery preserva LA COPIA (§93.4).

**Estado: RESUELTO.** Entrada 50 CERRADA y etiquetada (`entrada-50`):
hoja envuelta en árbol y circuitos (flip D4), colocación `public_id mod
capacidad`, y el barrido convertido en CONTRATO (`hallado.is_none()`).
`AUDITORIA.md` §117, §156-§158.

### 3.3 El contrato de lectura de cuentas no exige autorización — ⚠️ MEDIDO

`account_view(index)`, `balance_of`, `nonce_of` y `public_id_of` **toman un
índice y no piden credencial** — del OPERADOR por diseño (§129); el
titular tiene `account_view_authenticated` (49-A). Y los índices **ya
no** son secuenciales: colocación `public_id mod capacidad` (F3, §157)
— enumerarlos exige adivinar, y el contrato-test lo vigila.

⚠️ **ATENCIÓN: la eximente de este hallazgo CADUCÓ.** Hasta §197 este
apartado decía «no hay capa de red en este repositorio». **Hoy la hay**:
`zk-ssl-node` (JSON-RPC 2.0, axum). Lo que el nodo hace de verdad, leído
del código y no supuesto:

| método | credencial | veredicto |
|---|---|---|
| `zkssl_accountView` | **exige clave de VISTA** (49-A) | ✅ el RPC nació con el control de acceso puesto |
| `zkssl_publicId`, `zkssl_logEntries`, `zkssl_epochHead`, `zkssl_supply`, `zkssl_accountCount` | ninguna | **público por diseño** — el registro y los agregados son auditables a propósito. ⚠️ **CORREGIDO (§656)**: el `pending` de `zkssl_supply`, consultado tras cada entrada de `zkssl_logEntries`, da por diferencias el importe de cada envío y cobro; el proxy no debe exponerlo a terceros (sección 3.9) |
| `dev_*` | doble cerrojo: feature de compilación **y** `--dev` | un build de producción no los tiene |

**Lo que sigue abierto, dicho sin adornos**: el nodo **no tiene
autenticación, ni TLS, ni límite de tasa**, y lo único que hoy separa
«fuga hacia-el-operador» de «fuga hacia-terceros» es que escucha en
`127.0.0.1:8545` por defecto. **Publicarlo en `0.0.0.0` sin un proxy
delante es exactamente el escenario que este apartado advierte.** ⚠️ **§838: el tramo hasta el proxy.** La mandan al nodo el SDK (`crates/zk-ssl-sdk/src/lib.rs:192`, `:227` y `:267`) y el cli (`crates/zk-ssl-cli/src/pago.rs:150`, `cobro.rs:260` y `prenda.rs:163`), los dos por `ureq`; con una URL `https`, la cifra el TLS de `ureq`, que es `rustls` sobre `ring`, y su confidencialidad en tránsito descansa en un ECDHE clásico (X25519, P-256 o P-384): expuesta a «guardar ahora y descifrar después». Es la excepción del §106 para el transporte, declarada en el §838 de `AUDITORIA.md`; con una URL `http`, viaja en claro.

⚠️ **Y un segundo hallazgo, nuevo**: el nodo abre el ledger **sin cifrado
en reposo**. La capa tiene `open_encrypted` y `zk_ssl::crypto` desde hace
mucho, y el wallet del SDK duerme cifrado desde §199 — **pero el binario
del nodo no cablea ninguna clave**. Quien robe el disco de un nodo lee
los saldos. No es un fallo de la primitiva: es que no está conectada.

**Estado: parcialmente resuelto (vista autenticada), y abierto en
despliegue** (sin auth/TLS/límite de tasa; ledger del nodo en claro).
Backlog 49. `AUDITORIA.md` §93.1, §129, §157.
⚠️ §860: la 49-A está cerrada, y ese puntero cubría dos cosas. El ledger del
nodo en claro es la entrada 127 del `BACKLOG.md`. La falta de autenticación,
de TLS y de límite de tasa es un residuo de despliegue sin entrada: el proxy
delante, arriba; y para `zkssl_openAccount`, la 8, cerrada, con su viñeta en
la sección 2.

### 3.4 Espacio de los identificadores anti-doble-gasto — **no aplica**

La vía de producción es la de **dos fases**, y **no usa nulificadores**: un
envío cambia el saldo del pagador, luego su hoja, luego la raíz, de modo que
un reenvío presenta una raíz obsoleta y se rechaza. Y la raíz sola no bastaba:
el estado global se repite tras un reembolso o un ciclo A→B→A, y un recibo ya
aplicado volvía a valer. Desde el §654 la capa guarda además la huella de cada
prueba del titular ya aplicada y rechaza la que vuelva, y desde el §659 también
la que sólo cambie en nodos de más dentro de un lote de Merkle (§3.9).

⚠️ **Matiz (§715): la huella de los bytes no bastaba.** La guarda del §654 reconoce una prueba ya
aplicada por la huella de sus bytes, y el §659 le quitó una forma de esquivarla. Esa huella solo
identifica la operación si cada prueba tiene una sola escritura, y no la tenía. Sin el testigo ni
la clave, de los bytes de una prueba salían otros que verificaban igual: una longitud escrita con
más bytes de los necesarios, una fila o un grupo de más en una apertura, una capa FRI de más, o el
número de restricciones con un múltiplo de `2^32` sumado. Con una longitud escrita de más o con un
grupo de más en una capa FRI, una prueba ya aplicada volvía a valer en las mismas condiciones del
§654: las raíces que recurren tras un reembolso o un ciclo A→B→A. Lo que el §659 dio por «ahora ya
única» no lo era.

**Cerrado, en dos sitios.** El lector del fork y el canal del verificador exigen una sola
escritura: la longitud mínima, una fila por posición consultada y un grupo por posición plegada
(§715); el número de restricciones y el de capas FRI, en su rango (§711). Una prueba que no lo
cumple se rechaza con su causa. Y la capa guarda además, de cada prueba del titular aplicada, la huella
de su contenido: sus compromisos, que no dependen de cómo se escriban las aperturas. `ya_aplicada`
mira las dos huellas.

**Lo que no cierra.**

- **La huella del contenido no persiste.** Tras reiniciar el nodo, la guarda mira solo los bytes, y
  la defensa es el lector y el canal del fork.
- **Los recibos y los acuses** siguen identificando la prueba por sus bytes. Con el lector canónico,
  esos bytes son únicos en las formas medidas. En los recibos no se ha medido.
- **El cierre de fondo** sigue siendo el que dice el §659: el nonce en toda transición deudora, que
  pide el tren `zkssl/0.5`.
- **Los kits publicados hasta la 0.4.3** aceptan esas escrituras. Las rechaza el kit compilado
  desde el §715.

⚠️ **El problema existió y se retiró con su camino.** La vía de un paso
derivaba la posición del marcador del propio marcador, con colisión por
cumpleaños alrededor de **sesenta y cinco mil pagos** frente a los cuatro mil
millones que el árbol anunciaba.

⚠️ **Y fue evitado, no resuelto**: lo que sustituye al marcador es el
encadenamiento de raíces, que **exige un orden total** — el que un nodo único
da y un sistema distribuido no. Quien distribuya esto recupera el límite
intacto.

**Estado: cerrado en el nodo único, abierto para cualquier distribución.**

### 3.5 Definiciones duplicadas de la hoja — ⚠️ MEDIDO

Existen **tres** `native_leaf` en el árbol: en `circuit_settlement`,
`compliance_circuit` y `double_entry`.

**Misma estructura, distinta anchura de dominio**: la de producción toma la
identidad como `Digest` —256 bits—; las otras dos como `BaseElement` —64—.

⚠️ **Compartir nombre lo empeora**: invita a suponer que son la misma función.
Quien verifique una y dé por buenas las otras dos se equivoca, y nada en el
código se lo advierte.

**Estado: abierto.** Backlog 51. `AUDITORIA.md` §94.

### 3.6 La prueba no ata la posición del titular en congelados — ⚠️ MEDIDO, cerrado en la capa

En los cinco circuitos con fase de congelados (`burn`, `claim`, `claim_v2`, `send`,
`send_v2`) la columna del bit de esa subida (`COL_FBIT`) no está atada a la del
camino de cuentas (`COL_BIT`). Lo que el AIR prueba es que **alguna** posición del
árbol tiene hoja cero, no la del titular. Medido el 16-sep-2026 (`AUDITORIA.md`
§487): una cuenta congelada que probaba por su cuenta con el camino de la posición
vecina verificaba en los cinco y el aplicador la aceptaba en `apply_send`,
`apply_claim` y `apply_burn`. Exige la clave de gasto del propio titular y no rompe
la conservación: es un fail-open de una función de cumplimiento, no del dinero.

**Cerrado en la capa** el mismo día (S487): `validate_send`, `validate_claim` y
`apply_burn` rechazan con `AccountFrozen` antes de mirar la prueba; tres testigos
lo falsan. **Abierto en el AIR**: quien solo ve la prueba sigue sin poder saber si el
titular está congelado; atar la posición en los cinco circuitos es el arreglo B,
rotura de formato, con fecha detrás del AIR de E1 del RFC-0008.

No hay libro desplegado ni terceros a quien avisar; por eso el aviso es esta sección
y los asientos §487 y §488, no un GHSA. `AUDITORIA.md` §487, §488.

⚠️ **§670: el arreglo B está en el árbol y NO ata.** El §511 dio a los cinco circuitos
dos acumuladores de la posición —uno por subida— y una igualdad entre ellos, y dio el
atado por hecho. Medido en el re-triaje del segundo enjambre sobre `ed0f96b`: el
acumulador de la subida de congelados no suma el mismo bit que lee el multiplexor de esa
subida, así que la igualdad de acumuladores no obliga a que el camino recorrido sea el
del titular. Una traza de una cuenta congelada que recorre otra posición **verifica**
(`prove` y `verify` dan `Ok`), y la capa la para con `AccountFrozen`. El estado de esta
sección no cambia —cerrado en la capa, abierto en el AIR—, pero el arreglo del AIR
**no está hecho**, aunque el §511 lo diga: tiene que acumular el mismo bit que el
multiplexor lee, en los cinco, y cambia el formato de la prueba (tren `zkssl/0.5`).

⚠️ **§679: tampoco ata el acumulador de cuentas.** Medido en la sesión que decidió el
RFC-0018: `COL_IACC` suma `COL_BIT` en la fila +7 de cada ciclo de la subida de cuentas y el
multiplexor de esa subida lo lee en la fila +0 del siguiente, el mismo desfase. Una traza que
sube las cuentas por el titular congelado y pone en la fila +7 los bits de la vecina libre
también verifica. Misma clase y misma defensa: la capa la para con `AccountFrozen`. El arreglo
(E1 del RFC-0018, variante A medida) ata los dos acumuladores.

**§680: atado en el AIR del árbol.** E1 del RFC-0018 está construida en los cinco
circuitos: el paso de cada acumulador va en las filas del multiplexor de su subida y
suma el bit que el multiplexor lee. Los dos ataques medidos dejan de verificar en los
cinco (diez falsadores). El cable sigue diciendo `zkssl/0.4` hasta el corte del tren
`zkssl/0.5`, que sube la versión y emite sus vectores; hasta entonces lo que ata es el
código del árbol, no una versión publicada.

### 3.7 Una prueba malformada abortaba el proceso — ⚠️ MEDIDO, cerrado en las vías con sal

Al leer una prueba, `winter-utils` 0.13.1 (`read_many`) y `winter-crypto` 0.13.1
(`BatchMerkleProof::read_from`) reservan memoria con una cuenta sacada de los propios
bytes ANTES de comprobar que esos bytes existen. Una prueba malformada que declara una
longitud gigante hace que el proceso pida cientos de GB, y una reserva que falla **no
es un pánico**: el proceso aborta, y la red del pánico del nodo (§530) no lo ve.
Medido el 28-sep-2026 (`AUDITORIA.md` §575): los testigos del §519 que voltean un
byte de una prueba de prenda abortaron el nodo pidiendo 862.916.669.440 y
544.766.607.872 bytes, una vez en treinta corridas y otra en la VIVA de un bloque;
las pruebas llevan sal desde el §538, sus bytes cambian en cada corrida, y el byte
volteado cae a veces en una longitud. Lo podía disparar cualquiera que mandara una
prueba al nodo, y un sobre malformado hacía abortar al mando en vez de salir ROJO.

**Cerrado** en el §575 en los dos sitios que lo producían: `Proof::from_bytes` lee con
un lector acotado (fork de `winter-air`, `src/proof/acotado.rs`), y el lote de las
aperturas con sal (`zk-ssl-air/src/sal.rs`) se lee con sus cuentas acotadas, también
en las capas FRI. Cuatro testigos deterministas lo falsan. **Residuo, MEDIDO en el
§578: ninguno en las vías vivas.** Toda verificación que alcanza una prueba ajena lee
con sal: los quince `verify` de la capa, la pareja umbral de los custodios y los cinco
del kit, y dos testigos lo atan. Los verificadores SIN sal quedan en código que
ninguna vía viva alcanza —`range_check` y el `ComplianceAir` de `settlement-prover`,
experimentos, e `instrumento_edad`, sólo tests—; si una vía los alcanzara, leerían su
lote con el `BatchMerkleProof` de upstream, sin acotar. ⚠️ **CORRECCIÓN (§578):** el
§575 escribió aquí «los dos circuitos SIN sal (`range_check`, `governance`)» y «qué
entrada ajena llega a ellos está sin medir». La lista salió de una búsqueda recortada
y estaba mal por los dos lados: la gobernanza se verifica con sal
(`verify_threshold_pair`), y sin sal hay más, ninguno alcanzable. `winterfell`
upstream tiene el mismo defecto. **Reportado el 01-10-2026** por correo privado a su
mantenedor —el aviso privado de GitHub no está habilitado en `facebook/winterfell` y su
política hereda la de Meta, que exige una cuenta de Facebook—, con el arreglo ofrecido como
PR y sin reproductor; pendiente de respuesta. `AUDITORIA.md` §575, §578, §625.

### 3.8 Una firma acreditaba un entero módulo `p` — ⚠️ MEDIDO, cerrado en el núcleo y en el mando

El núcleo compone sobre `F_p`, `p = 2^64 - 2^32 + 1`, y dos lecturas reducían en silencio:
`as_digest` embebe un `u64` reducido módulo `p`, y `element_from_bytes` leía ocho bytes con
`BaseElement::new`, que también reduce. Así una cabeza firmada fijaba cada entero sólo **módulo
`p`**, y el mando razona sobre el entero. Medido el 01-10-2026 (`AUDITORIA.md` §640) sobre vectores
reales mutados en un solo campo: la misma firma acreditaba una cabeza con `n` y otra con `n + p`; el
sobre de completitud que nombra al operador —«NO RESUELTA EN LA VENTANA»— pasaba a «ventana ABIERTA»
con la misma firma, de modo que **el acusado elegía el veredicto** escribiendo `n + p`; una extensión
con `mmrSize + p` **colgaba el mando** (la partición del MMR desbordaba para tamaños mayores que
`2^63`); y el cero escrito como `p` pasaba en la referencia y se rechazaba en la segunda
implementación.

**Cerrado** en el §640 (RFC-0016): `u64_canonico` es el único productor de la regla «un `u64` escribe
un elemento sólo si es menor que `p`»; `element_from_bytes` la aplica a cada elemento y el mando a
cada `u64` del sobre, antes de recomponer y antes de la firma; la partición del MMR es total. Cuatro
vectores negativos lo atan, y la segunda implementación los rechaza con el mismo texto. Ningún KAT ni
ninguna cabeza custodiada se mueve. **Residuo**: el lector de `QUANTITY` del cable y el recompositor
del testigo siguen leyendo sin la regla —lo que firman y comparan es el digest, no el entero—, y el
kit publicado `arqueo-verify-v0.3.0` es anterior: acepta los cuatro negativos hasta una release nueva. La
`arqueo-verify-v0.4.0` (§658) los rechaza, medido con el kit descargado. ⚠️ §825: el residuo de arriba ya no es el de hoy. El recompositor del testigo lee con la regla desde el §775, y el SDK también, en el `logSeq` de la constancia; el lector de `QUANTITY` exige la escritura mínima desde el §662 y deja la cota a quien lo usa (`zk_ssl_wire::cantidad_canonica`). Al aceptarse el RFC-0016, la release vigente es la `arqueo-verify-v0.4.3`, publicada con su aviso (§818), que rechaza los cuatro igual, medida con su binario publicado (§825); la 0.4.2 (§671), publicada y ya no vigente, también. Y el RFC-0016 está ACEPTADO desde el §825, con su residuo declarado en su cabecera.

### 3.9 Ocho hallazgos de un análisis con agentes — ⚠️ MEDIDO, siete cerrados y uno acotado

En el §637 una sesión de Claude Code analizó el árbol con un enjambre de agentes
([`doc/blueprint-v2.md`](doc/blueprint-v2.md)) y entregó al autor, en privado, ocho hallazgos
abiertos en `main`. Se corrigieron antes de describirlos aquí, como pide la sección 5. Tres se
reprodujeron con tests antes del arreglo; cada arreglo lleva falsadores que caen con él desactivado.

| hallazgo | qué rompía | prioridad | cerrado en |
|---|---|---|---|
| El hexadecimal del cable y del kit se troceaba por bytes de un `&str` | una sola petición sin credencial con un carácter multibyte paraba el nodo (PARADA); el kit salía con 101 en vez de ROJO; `+` y mayúsculas se aceptaban | P0 | §650; ⚠️ en los `u64` (QUANTITY), el §662; en el testigo y el SDK, el §775 |
| La guarda de forma no miraba la marca del meta | una prueba oculta con el meta vaciado hacía entrar en pánico a la capa (con el candado del nodo tomado) y al kit | P0 | §651 |
| Las semillas de la ocultación eran de 64 bits | quien ve una prueba despejaba cualquier columna constante del testigo —la clave de gasto, la de un custodio— con unas 2^74 compresiones Blake3 (ESTIMADO) | P0 | §652 |
| `applySend` y `applyClaim` guardaban el importe y el saldo que mandaba el cliente | un titular rompía la conservación en la contabilidad `u64` y el libro no volvía a abrir | P0 | §641 (RFC-0017) |
| `Proof::from_bytes` aceptaba bytes de cola | la huella de una prueba no identificaba una operación | P1 | §653 |
| El reenvío sólo se paraba por la igualdad de raíces | tras un reembolso o un ciclo A→B→A, un recibo ya aplicado volvía a valer | P1 | §654 |
| `zkssl_supply` publica sin credencial el total en tránsito | por diferencias da el importe de cada envío y cobro | P1 | **acotado**: ver abajo |
| Un fallo del almacén no paraba el nodo | la memoria podía firmar raíces que el disco no tiene | P2 | §655 (c); ⚠️ §815: el §655 paró el despacho y no el latido, que siguió firmando en PARADA; y `zkssl_publishConsumo` devolvía el fallo como una negativa. Cerrado entero en el §815 (3.15) |

⚠️ **CORRECCIÓN (§815), citada y no borrada: «siete cerrados» no eran siete.** El fallo del almacén
lo cerró el §655 en el despacho, no en el latido. Con el nodo en PARADA, `latido::latir` no miraba
la PARADA y siguió firmando cabezas desde la memoria, la que va por delante del disco: medido en
privado sobre `4d8017d`, cabezas del `seq` 9 con el disco en el `seq` 8, que es justo lo que la fila
dice que se cerró. Y `zkssl_publishConsumo` devolvía el fallo del almacén dentro de un resultado, sin
PARADA. Hasta el §815 eran seis cerrados, uno a medias y uno acotado; desde el §815, siete. El
comentario de `despachar` («el nodo no sigue sirviendo») y el del test del §655 llevan su marca. Lo
cuenta la 3.15.

⚠️ **Lo que conviene hacer si se usó el sistema con claves propias.** Las pruebas ocultas emitidas
entre el §538 y el §652 protegen la clave de gasto y las de los custodios sólo a unas 2^74
operaciones clásicas (ESTIMADO; con Grover, unas 2^32 iteraciones). El ataque es fuera de línea y
una prueba emitida no se puede volver a ocultar: **quien haya publicado pruebas con claves propias
debe rotarlas**. La exposición conocida son despliegues de desarrollo y de prueba, con claves ya
públicas: el nodo sin la feature `dev` no arranca (§637).

⚠️ **`zkssl_supply`, acotado y no cerrado.** El campo `pending` es normativo (`spec/RPC.md`,
`spec/openrpc.json`): exigir la credencial del operador o quitarlo cambia el cable, y se hará con la
próxima versión que ya haga falta, no subiéndola sólo por esto. **Mientras tanto, el proxy que
publique el nodo no debe exponer `zkssl_supply` a terceros**: la fila de la tabla de la sección 3.3
que lo llamaba «público por diseño» queda corregida. Quien ve una prueba de envío o de cobro ya ve
su importe (son entradas públicas, RFC-0009); lo que el método añadía es dárselo a cualquiera.

Lo que queda abierto de estos ocho, y es decisión del autor: las partes (a) y (b) del fallo del
almacén (anotar antes de verificar, y cachear la pareja de recepción), el cambio de cable de `zkssl_supply`.
⚠️ §815: la (a) cambió de forma a medias. Desde el §815 la hoja se anota TRAS juzgar (3.15); pero el
contador de recepción sigue reservando el `rx` con su `fsync` ANTES de verificar (`recibir`;
RFC-0010, D-E), así que pagar un `fsync` por una petición sin credencial antes de verificar sigue
siendo cierto. Cada petición que la capa juzga paga ahora dos, el del contador antes y el de la hoja
después, también la de un tercero sin cuenta, porque un rechazo deja hoja (deducido de leer el
orden; la latencia no está medida). Y era esa hoja la que le daba a un tercero un ROJO contra el
operador honrado, hasta el kit 0.4.3 (3.15).
La versión nueva del kit está publicada: `arqueo-verify-v0.4.0` (§658); la `v0.3.0` sigue afectada
por los dos primeros y por la cola. ⚠️ **Y una variante de la cola, por dentro (§659)**: nodos de más
dentro de un lote de Merkle no movían la raíz, así que la prueba verificaba con otros bytes y el
índice de pruebas aplicadas del §654 se esquivaba; la capa y el kit los rechazan desde el §659, y la
vigente era `arqueo-verify-v0.4.1` (§661), y después `arqueo-verify-v0.4.2` (§671), que corrige además los §662 a §664; hoy es `arqueo-verify-v0.4.3` (§818), que corrige además el F3 de la 3.15 (§815). ⚠️ §818: decía «hoy es `arqueo-verify-v0.4.2` (§671)». `AUDITORIA.md` §637, §641,
§650 a §656.

### 3.10 Las claves de custodio y de gobernanza: de cuatro elementos en el cable, de uno en el libro de `dev` — ⚠️ CONSTRUIDO (§683, §684, §701) y DECLARADO

Hasta el §684, la identidad de un custodio (`derive_custodian_id`, `circuit_threshold.rs`) y
la de un miembro de la gobernanza (`derive_governor_id`, `circuit_governance.rs`) se
derivaban de una clave de **un solo** elemento de Goldilocks: unos 64 bits. La identidad es
pública, así que agotar el espacio de una clave es un ataque fuera de línea de unas 2^63
evaluaciones (ESTIMADO; con Grover, unas 2^32 iteraciones). La emisión pide dos custodios
(2-de-N), así que hacen falta dos claves. Lo declaró el §670, y el RFC-0018 E2 lo cierra en
el formato, en tres piezas:

- **Las derivaciones anchas** (§683): `derive_custodian_id_wide` y `derive_governor_id_wide`,
  el molde de `derive_public_id_wide` de la clave de gasto (entrada 15, §82): el dominio y
  una clave de cuatro elementos en el mismo `native_merge`.
- **El umbral del cable** (§684): `circuit_threshold_single_nullifier`, el único circuito de
  autoridad que verifica la capa —para los custodios y para la gobernanza—, lleva la clave en
  cuatro columnas y ata los cuatro elementos a la identidad.
- **La capa** (§701): un libro constituido con claves anchas emite, emite a un pendiente,
  recupera, congela y rota sus custodios; y una clave de un solo elemento —lo único que
  alcanza la búsqueda de 2^63; aquí, incluso con el primer elemento verdadero de la clave
  ancha y el camino de su hoja— no autoriza, porque su identidad es otra
  (`una_clave_de_un_solo_elemento_no_autoriza_sobre_un_conjunto_ancho`). Forzar una clave
  ancha cuesta unas 2^255 evaluaciones (ESTIMADO, por el tamaño del `Digest`, como el
  RFC-0018; con Grover, unas 2^128 iteraciones, por la misma cuenta que los 2^32 de arriba).

Lo que sigue valiendo un elemento, y se dice aquí:

- **Una clave estrecha sigue valiendo, rellenada con ceros**: da la misma identidad (§683).
  Un libro constituido con claves de un elemento conserva sus raíces y sus 64 bits. Sus
  custodios pueden rotar a claves anchas con la firma de su gobernanza
  (`un_libro_de_claves_estrechas_rota_sus_custodios_a_claves_anchas`); su gobernanza no,
  porque es inmutable: una gobernanza ancha la da sólo un libro nuevo (RFC-0018, D-B y D-C).
- **El nodo constituye un libro con las raíces que se le pasan** (§784, ARQ-02):
  `--custodian-root` y `--governance-root`, las de la ceremonia, en la escritura canónica del
  cable y distintas del digest cero, dichas al arrancar. En un libro que ya existe, la de
  gobernanza se compara y, si no es la suya, el libro no abre. Sin las dos banderas, con la
  feature `dev` —la de la compilación por defecto— abre la capa con las raíces de la suite, de
  claves de un elemento escritas en `crates/zk-ssl/src/tests_support.rs`, y lo dice; sin `dev`
  no arranca. El canon compila el nodo sin `dev` en cada sello. ⚠️ §784: hasta aquí decía «El
  nodo no constituye libros» y que las dos banderas seguían pendientes.
- **El cable sigue diciendo `zkssl/0.4`** hasta el corte del tren (RFC-0018; `spec/RPC.md`,
  §699). Los vectores de rechazo llevan la raíz de custodios de la suite, y los publicados no
  se reescriben.
- **Los otros siete circuitos que llevan una clave de autoridad** —`ThresholdAir`,
  `SingleThresholdAir`, `GovernanceAir` y los de emisión, emisión a pendiente, congelación y
  recuperación de la vía antigua— siguen con un elemento. No van por el cable: ni la capa ni
  el kit los verifican (el censo del RFC-0009), y no prueban la pertenencia a un conjunto
  ancho.

### 3.11 El nivel de las pruebas, y el azar del sistema — ⚠️ MEDIDO y DECLARADO (§697, §698, §708)

**Lo medido es clásico.** Las opciones de producción, las de la capa y las del kit, son 42
consultas, blowup 16, molienda de 21 bits y extensión cuadrática (`proof_options()` en
`crates/zk-ssl/src/lib.rs`; la edad usa las mismas, y un test lo exige). Sobre la forma oculta
de cada prueba, la función de `winter-air` (`proof/security.rs`, la de upstream) da **127 bits
conjeturados** —de 112,8 a 116,8 según la longitud con el término DEEP, que su fórmula no
descuenta (abajo)— y, demostrables, **59 en UDR** (decodificación única) en todas, y en
LDR (decodificación en lista) lo que dé la longitud de la traza:

| traza oculta | LDR | conjeturada, con el DEEP | pruebas |
|---|---|---|---|
| 128 filas | **88** | 116,8 | las dos aperturas del reembolso y el umbral (T = 64) |
| 512 filas | **84** | 114,8 | la subida de congelados (T = 256) |
| 1.024 filas | **82** | 113,8 | el crédito, las subidas de emisión, de emisión a pendiente y de recuperación, la auditoría y las cuatro del kit de longitud fija (T = 512) |
| 2.048 filas | **80** | 112,8 | envío, cobro y quema (T = 1.024) |

**La edad baja con `m`.** Su traza oculta tiene 2^(m+4) filas, con `m` el logaritmo del número
de pendientes que cubre, y pierde dos bits de LDR cada vez que la traza se dobla:

| `m` | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| LDR | 88 | 86 | 84 | 82 | 80 | 78 | 76 | 74 | 72 | 70 | 68 |
| con el DEEP | 116,8 | 115,8 | 114,8 | 113,8 | 112,8 | 111,8 | 110,8 | 109,8 | 108,8 | 107,8 | 106,8 |

| `m` | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| LDR | 66 | 64 | 62 | 60 | 58 | 56 | 54 | 52 | 50 | 48 | — |
| con el DEEP | 105,8 | 104,8 | 103,8 | 102,8 | 101,8 | 100,8 | 99,8 | 98,8 | 97,8 | 96,8 | — |

Conjeturada 127 y UDR 59 en todas; la fila «con el DEEP» es la conjeturada contando el término
DEEP (abajo). Con `m = 24`, que el enunciado admite, la prueba no se puede generar hoy.
⚠️ **§717:** desde el §717 el enunciado admite `m` de 3 a 13 (`zk_ssl_air::M_MAX`), y el juez
rechaza las demás antes de leer la prueba. La segunda tabla, de 14 a 24, queda como lo que da la
función para longitudes que el juez ya no acepta, y la frase de `m = 24` dejó de ser cierta. El tope
sale del coste del juez y de las cotas de la puerta del RFC-0007, no del nivel: con `m = 13` la LDR
es 68, y el piso de LDR y el umbral de UDR siguen sin decidir. Las cifras de la segunda tabla ya no
las fija un test: las ata a la réplica de la función el ATADO F de `tools/check_publicadas.py`.

Lo fijan tests desde el §697: en las familias de longitud fija, conjeturada ≥ 127, LDR ≥ 80 y
UDR = 59 sobre pruebas ocultas reales; en la edad, el piso de cada `m`. Desde el §839, cada fila
de la primera tabla tiene su prueba en ese test —la de 512 filas era derivada, no medida—, y un
censo exige que cada AIR del cable, salvo la edad, tenga una de esas longitudes. **El nivel que se
declara —el piso de LDR, el umbral de UDR y el rango de `m`— lo decide el autor**
(`BACKLOG.md`, entrada 116); hasta entonces, lo que vale son estas tablas. La conjeturada se
apoya en conjeturas sobre la proximidad a códigos Reed-Solomon que no están demostradas; la
demostrable, no. Las dos son lo que devuelve la función de upstream sobre la forma de la
prueba, y nadie ajeno al proyecto las ha contrastado con la construcción oculta del fork (H7).

**La conjeturada no descuenta el término DEEP (§708).** Su fórmula, la de `winter-air`, es
min(bits del campo, consultas · log2(blowup) + molienda) − 1, con el tope de 128 que pone el hash:
pone como término de campo el campo entero, 128 bits con la extensión cuadrática, y ningún término
que crezca con la longitud L de la traza oculta. La demostrable sí cuenta uno, y no es de
proximidad —de lo que tratan las conjeturas— sino de grado: el del punto fuera del dominio (DEEP),
un error de ((blowup + 1) · (L + 1) + L − 1)/|E|, con |E| el tamaño de la extensión, que la función
cuenta como 2^128, el grado de las restricciones acotado por el blowup, y en decodificación única,
sin el tamaño de lista de la LDR. Vale 116,8 bits con 128 filas y casi uno menos cada vez que L se
dobla —112,8 con 2.048, y 106,8 en la edad con `m = 13`, la mayor que el juez acepta desde el
§717 (96,8 con `m = 23`, antes del tope)—, y, puesto en lugar del campo, es el menor de los
términos de la fórmula: las consultas dan 189 bits y el hash pone 128. Con él, la
conjeturada no pasa de ese término, y esa es la cifra «con el DEEP» de las tablas: el término
mismo, sin el −1 con que la fórmula resta un bit a su mínimo (sus 127 son 128 − 1); con ese −1,
sería un bit menos. El otro término de campo, el de ALI, no cambia con L: con el lote lineal de
producción es 1/|E|. Esas cifras son una derivación, no lo que devuelve la función; las fija un
test (`tests_nivel` en `crates/zk-ssl-air/src/lib.rs`), que contrasta su réplica de la fórmula y
del término con la función real y con su fuente. Los 127 son lo que la función devuelve, y lo que
siguen exigiendo los tests del §697.

Los «127 conjeturados frente a 29-63 demostrables» y los «36,7 KB frente a 125,6 KB» que
publicaban varios documentos hasta el §698 son del circuito de comparación de
`FIVE_BACKENDS.md` §4, con 32 consultas y sin molienda ni ocultación, no de estas pruebas. Lo
que costaría llevar las de producción a 128 bits demostrables no está medido. ⚠️ §806: y esos dos
tamaños tampoco son lo que da hoy aquel circuito: remedido el 2026-10-14 sobre `b6491ef`, con el
test de su tabla, da 39.048 y 127.274 B.

**Frente a un adversario cuántico: no medido.** Que el sistema de pruebas sólo use hashes
quiere decir que no tiene supuestos de curva. No quiere decir que tenga un nivel cuántico
conocido: ninguna cifra de este apartado lo es, ningún parámetro se eligió pensando en uno, y
ese nivel no se ha medido.

**El azar del sistema es un supuesto de confianza.** Lo que tiene que ser impredecible en la
capa, el SDK y el medio sale del generador aleatorio del sistema operativo, en ocho llamadas y
sin una abstracción común (siete hasta el §702, que añade la sal del KDF de reposo):

- las sales de cada hoja de Merkle y las dos semillas de la ocultación de cada prueba
  (`OsRng`, `crates/zk-ssl-air/src/sal.rs`);
- la clave de gasto de `Wallet::random` y la sal del pendiente de `random_salt`
  (`rand::thread_rng`, `crates/zk-ssl-sdk/src/lib.rs`);
- los nonces del cifrado en reposo del libro y del keystore (`OsRng`,
  `crates/zk-ssl/src/crypto.rs` y `crates/zk-ssl-sdk/src/keystore.rs`), y la
  sal del KDF de reposo de los dos (`OsRng`, `Kdf::nuevo` en `crates/zk-ssl/src/crypto.rs`);
- la aleatoriedad de la firma ML-DSA del medio en su modo con sal (`getrandom`,
  `crates/zk-ssl-medio/src/nota.rs`).

Si ese generador es predecible o se repite, la ocultación no protege el testigo, la clave de
gasto se puede adivinar, los nonces pueden repetirse y dos ficheros cifrados pueden compartir
sal. Ningún test lo comprueba, ni puede. Fiat-Shamir no usa azar: la moneda sale de lo que la
prueba compromete.

### 3.12 El guardián del índice arrancaba a veces en tmpfs — ⚠️ MEDIDO, cerrado en Linux (§740)

El guardián del índice XMSS se niega a arrancar donde `fsync` no persiste, porque un contador que
se pierde al apagar reabre en 0, y con él las hojas ya gastadas vuelven a parecer libres. Hasta el
§740 lo decidía sólo por tiempos, con la media de veinte escrituras, y fallaba abierto: una sola
escritura lenta bastaba. Medido en el contenedor de una sesión (`AUDITORIA.md` §740): de 2 a 11
aperturas de cada 2000 arrancaban en `/dev/shm`, en reposo como con carga, y el canon del autor
salió ROJO el 08-10-2026 porque el cofirmante del testigo arrancó allí una vez. El único test que
debía ejercitar la negativa en el guardián se saltaba siempre con el `df` de GNU.

**Cerrado en Linux** en el §740: el guardián lee el tipo del sistema de ficheros del descriptor en
`/proc/self/mountinfo` y se niega en `tmpfs`, `ramfs`, `devtmpfs` y `rootfs` sin medir nada, en la
carpeta y en el fichero del contador; y la medida de `fsync` queda como segunda red, con la mediana.
Medido: 8000 de 8000 aperturas en `/dev/shm` negadas por el tipo, y ninguna de 538 000 arrancó con
la mediana sola. **Residuo**: fuera de Linux sigue habiendo sólo la medida; en Linux, un sistema que
no se llama como los de la lista y no persiste sólo lo coge la medida si `fsync` cuesta lo mismo que
no hacerlo, y la medida mira la carpeta, no un contador que sea un montaje propio. Y nada de esto
protege de un disco que confirma escrituras que siguen en una caché volátil, ni de un corte de
corriente, que no se ha medido. El guardián no está en el kit publicado: no lleva aviso ni release.

### 3.13 Abrir una cuenta no comprobaba que la identidad fuera nueva — ⚠️ MEDIDO, cerrado en la capa (§802)

`open_with_id`, por donde pasan todas las vías que abren —`zkssl_openAccount`, `dev_openSeeded`, las
aperturas estrecha y ancha de la capa—, no miraba si la identidad era nueva: sondeaba la posición
`public_id[0] mod capacidad` y abría en el primer hueco. Medido el 14-10-2026, en privado y con cuatro
tests fuera del árbol (`AUDITORIA.md` §802): un tercero **sin la clave** abría una segunda cuenta con la
identidad de Bob y con su propia vista y su propia sal (`Ok`, índice 700774416 frente al 700774415 de
Bob); tras una recuperación hacia una clave nueva, quien conservara la clave **vieja** —el ladrón, si
la recuperación era por robo— abría otra cuenta con la identidad retirada y cobraba ahí un pendiente de
7.000 que estaba en vuelo hacia ella; la clave nueva no lo cobra en la cuenta recuperada
(`StaleState`); y el mismo pendiente no se cobra dos veces entre duplicados (el segundo `apply_claim`
da `StaleState`, y el suministro de 500.000 cuadra). No crea dinero: lo que rompía es a quién llega.

**Alcance.** Por el cable, un tercero sin la clave ocupaba la identidad de otro y una plaza del tope de
cuentas, sin poder cobrar nada en ella. El robo de un pendiente en vuelo exigía una recuperación, y la
recuperación solo está en la biblioteca y en sus tests: ningún nodo llegaba a ese caso.

**Cerrado** en el §802, en la capa, con dos reglas: una identidad —el `public_id` entero, sus cuatro
elementos— tiene a lo sumo una cuenta en la vida del libro, y la que una recuperación retira no vuelve a
tener ninguna. Abrir rechaza las dos con `IdentityUnavailable` (`-32000` en el cable, AM05 en ISO
20022) antes de mutar nada; recuperar hacia una identidad con cuenta o retirada —también deshacer
X→Y→X— se rechaza antes de verificar, sin gastar cupo, contador ni nonce. Las retiradas se escriben
en el mismo lote que la recuperación (`retir:`, con su raíz `root:retir`), selladas si el libro está
cifrado, y se exigen al reabrir: un libro no abre si faltan o sobran, y uno cifrado tampoco si se
sustituyen; la instantánea pasa a la v9 y las lleva; la migración del KDF las vuelve a sellar, y la de
B13/B14 no las toca. De los veintidós tests nuevos, veintiuno lo falsan, cada uno contra su mutante, y
uno fija la medida de la D-5 de `doc/ROTACION_DE_CUENTAS.md`. Hasta el §802, una semilla repetida en
`dev_openSeeded` o en el sandbox de la CLI abría, sin error, otra cuenta con la misma identidad; los dos
pasan a la vía que devuelve el rechazo, porque con el `expect` de `open_account_wide` la puerta nueva
sería un pánico —en el nodo, con el candado tomado, y la PARADA, que ya daba `AccountLimitReached` con
el libro lleno—.

**Residuo**:

- las cuentas duplicadas que ya estén en un libro siguen ahí: el libro abre y el índice las colapsa.
  Y lo peor de ellas: si el duplicado B(X) lo abrió quien tenía la clave de X y después se recupera la
  cuenta legítima X→Y, B conserva X y cobra lo que esté en vuelo hacia X —el robo de antes del §802, con
  un duplicado de antes del arreglo—. Medido en privado: la recuperación retira X, y aun así el cobro
  en B da `Ok` y su saldo queda en 7.000. Solo lo alcanza un libro con duplicados y recuperaciones, y
  la recuperación no está en el cable;
- la integridad de las retiradas en reposo es la del libro: en uno cifrado, una retirada borrada,
  añadida o sustituida no abre; en uno en claro, quien escribe en el disco rehace `root:retir` —la
  cadena es `native_merge`, pública— y una retirada sustituida abre (medido: la identidad retirada
  vuelve a abrir cuenta). Es lo que ya puede hacer con el resto de un libro en claro, y las retiradas no
  van bajo la cabeza, así que ningún testigo lo ve;
- con una copia anterior del libro y acceso al disco, la hoja rotada de después pegada sobre el
  registro de antes abre, cifrado o en claro: `load` exige las retiradas contra el registro, pero no
  cruza `root:state` con la última entrada que mueve el árbol de cuentas. Medido en privado: el libro
  reabre con cero recuperaciones y la identidad retirada vuelve a abrir. Es lo mismo que un rebobinado
  entero, que deshace la recuperación entera y devuelve la cuenta a la clave vieja; el cruce queda
  aparte;
- una instantánea v9 en claro lleva las retiradas sin ninguna raíz que el receptor pueda comprobar:
  `SnapshotInfo` no la trae, y una cola cambiada importa (medido: la identidad retirada vuelve a abrir
  en la capa importada). Solo la cifrada las protege (⚠️ §817: y, hasta el §817, ni ella: con su frase,
  la importación aceptaba también una en claro, sin mirar la clave; §3.16);
- un libro con recuperaciones escritas antes del §802 no abre: sin sus retiradas se ve igual que uno al
  que se las han borrado en reposo, y abrirlo vacío devolvería el robo. Solo lo fabrica la biblioteca,
  porque la recuperación no está en el cable; y lo mismo una instantánea v3..v8 con recuperaciones.
  Es lo que el autor eligió frente a abrirlos vacíos (D-1 del §802);
- la ocupación anticipada se agrava: quien conozca una identidad antes de que su dueño abra la cuenta
  la ocupa con su vista y su sal, y el dueño tiene que cambiar de clave. Y bloquea una recuperación:
  quien conozca la identidad nueva antes de que se aplique la ocupa, y la recuperación legítima sale con
  `IdentityUnavailable`; hoy sin consecuencias, porque la recuperación no está en el cable. Cerrarlo
  exige probar la posesión de la clave al abrir, un circuito y un cambio de cable: el autor lo dejó como
  residuo declarado y lo lleva al corte `zkssl/0.5`, en la entrada 119 del `BACKLOG.md` (D-2 del §802);
- la causa no tiene prueba portable: «tiene cuenta» exigiría abrir la hoja, y las retiradas no van bajo
  la cabeza;
- una sola causa no dice cuál de las dos reglas saltó, aunque recorrer con `zkssl_publicId` la cadena
  de sondeo deja inferirlo en el caso común (no para la identidad nueva de una cuenta recuperada, que
  se queda en la cadena de sondeo de la vieja);
- un pendiente hacia una identidad retirada solo vuelve al emisor por reembolso, al caducar, y el
  reembolso tampoco está en el cable (la D-5 de `doc/ROTACION_DE_CUENTAS.md`).
  ⚠️ §860: la D-5, decidida por delegación (la 52 del `BACKLOG.md`): quedan varados hasta ese
  reembolso. Y DEDUCIDO, sin test: lo que una cuenta envió por v2 antes de que la recuperaran ya no
  le vuelve por reembolso —`apply_refund` exige que la cuenta del emisor conserve la identidad del
  compromiso, y da `PendingMismatch` (`crates/zk-ssl/src/two_phase.rs:710-714`)—: solo sale si el
  receptor lo cobra. Falla cerrado: no crea ni mueve dinero.

Se redactó en privado y se publica de una vez con el código, como pide la sección 5. No hay libro
desplegado ni terceros a quien avisar: el aviso es esta sección y el asiento §802, no un GHSA. El kit
publicado no cambia: no depende de la capa, y un sobre de rechazo con esta causa lo rechaza por su
nombre. `AUDITORIA.md` §802. ⚠️ §863: y lo muestra un vector con ruta nueva, como pide la R6 f del RFC-0021 a un arreglo que cambia un veredicto dentro de `0.N.x`: `spec/vectors/nodo/abrir-identidad-con-cuenta.json`, abrir por el cable una identidad que ya tiene cuenta, que el canon juzga con un test del nodo (la 90 del `BACKLOG.md`).

### 3.14 El libro cifrado migrado desde la versión 1 guardaba en sus ficheros lo sellado con la clave v1 — ⚠️ MEDIDO, cerrado en la biblioteca (§812); su carrera, medida y cerrada en el §816

Desde el §702 (RFC-0001, E2) un libro cifrado de la versión 1 —clave `SHA-256(dominio_v1 ‖ frase)`,
sin sal ni coste— se migra a la 2 al abrirlo con su frase. La migración sobrescribía cada valor en
un lote sobre el mismo `sled` y hacía `flush`: lo que se lee cambiaba entero o nada, pero sled es un
log, y los valores sobrescritos siguen en los ficheros del directorio hasta que sled reaprovecha su
segmento, lo que puede no pasar nunca. Medido en el §812 sobre sled 0.34.7, el del `Cargo.lock`,
buscando los bytes de cada valor sellado en los ficheros del libro: con un libro de dos cuentas, sus
25 valores sellados con la clave v1 seguían, los 25, en los ficheros tras el lote, tras 200 lotes
más y tras tres reaperturas (sobre el código de antes del §802; desde él, el mismo libro lleva 26,
con `root:retir`, y el lote en sitio deja los 26); con uno de 702 cuentas, 1425 de 1425 tras el
lote, 1356 tras 200 lotes más, y ninguno tras tres reaperturas, cuando sled rehízo sus segmentos (si
los sobrescribió o los devolvió como bloques libres al sistema de ficheros, no se midió). Con el de
dos cuentas siguieron los 25 tras tres reaperturas: no se puede contar con que reabrir limpie.
Cuánto dura el resto depende de lo que el libro escriba después: en uno pequeño o quieto, no se va.
El test del §702 miraba lo que sled devuelve, no los ficheros, y ningún documento lo decía.

**El ataque.** Quien se lleve el disco —o una copia del directorio— de un libro migrado prueba
frases contra ese resto a la velocidad de un SHA-256, unos 159 ns por intento (§702), en vez de un
Argon2id de 64 MiB; la etiqueta de XChaCha20-Poly1305 le confirma la buena, y esa frase abre también
la versión 2, cuya sal está en claro. El endurecimiento del RFC-0001 no protegía esos discos. **La
ventana**: los libros que migró el código desde `2ac6db4` (§702, 07-10-2026) hasta el §812.

**Lo que lo cierra** (§812, RFC-0001 E4, redactada en privado con este arreglo): la migración ya no
escribe en sitio. Copia lo vivo, sellado con la clave nueva, a un directorio hermano
(`<libro>.recifrando`), lo verifica en claro leyéndolo del disco, lo cambia de nombre por el libro y
borra el viejo entero. Una muerte a mitad de la migración la resuelve la apertura siguiente, con la
misma frase, sin dejar nada de la v1 en el directorio del libro
(`una_muerte_en_la_migracion_v1_no_deja_en_el_directorio_lo_sellado_con_v1`, con una muerte al
acabar cada paso). La misma pieza es `SovereignLayer::recifrar`, que cambia la frase y la sal del
libro entero; si un recifrado muere antes de su compromiso, la apertura vuelve atrás y el libro
sigue entero con la frase vieja, con lo sellado con ella: el recifrado hay que repetirlo; y si falla
con un error pasado el compromiso, el `.retirado`, si sigue en el disco, guarda lo sellado con la
vieja hasta la apertura siguiente, que lo borra (⚠️ §816: hasta el §816, sólo si nadie abría el libro a la
vez; abajo, «La carrera de la E4»). Las identidades retiradas de la §3.13 (`retir:{k}`
y `root:retir`) van selladas, y se copian y se verifican como lo demás
(`recifrar_conserva_las_identidades_retiradas`). Medido con mutantes: si la migración vuelve a ser
un lote en sitio, `la_migracion_v1_no_deja_en_los_ficheros_lo_sellado_con_v1` cae con 26 de 26
valores v1 en los ficheros (25 de 25 antes del §802); con la pieza nueva no queda ninguno en los
ficheros del libro ni de sus hermanos, también en el de 702 cuentas.

**El remedio de un libro ya migrado**: `SovereignLayer::recifrar` **con una frase nueva**. La
biblioteca no distingue un libro migrado —aquella migración sólo escribió `meta:kdf`—, así que es a
mano. Deja el directorio limpio de lo sellado con la v1 y con la frase vieja
(`un_libro_migrado_por_el_702_se_limpia_recifrandolo`). Con la misma frase el directorio también
queda limpio, pero las copias hechas desde la migración y los bloques libres siguen dando la frase
v1 a la velocidad de un hash, y esa frase abre el libro.

**⚠️ §816: la carrera de la E4, y lo que la cierra.** Medido en privado sobre `b6994c3` —el §812, ya en
`main`— por un revisor, con dos hilos y pausas inyectadas en la biblioteca: la apertura miraba los
hermanos —el cerrojo de sled de cada uno, tomado y soltado en el acto; sin hermanos, ninguno— y
después actuaba y llamaba a `sled::open`, que crea el libro si no existe. Nada retenía un cerrojo
entre mirar y actuar. (a) Sin hermanos, una apertura que miraba y se paraba, y un recifrado que
llegaba en medio a su compromiso —`rename(P, V)`—, dejaban un libro VACÍO en el sitio del libro, que
la apertura abría `Ok` sin saldo; el paso 6 del recifrado fallaba con «la apertura siguiente lo
termina», y la siguiente rehusaba: el libro, entero en su `.recifrando` y en su `.retirado`, no
volvía a abrir sin una persona. (b) Con el libro y su `.retirado` de una muerte tras el paso 6, la
apertura borraba el `.retirado` NUEVO del recifrado y creaba un libro vacío, y la apertura siguiente
con la frase que nombraba el error borraba el `.recifrando`, la última copia. (c) Con el libro y su
`.recifrando` de una muerte antes del compromiso —medida por un revisor del arreglo, también sobre
`b6994c3`—, la apertura borraba el `.recifrando` NUEVO y creaba un libro vacío; el paso 6 fallaba
diciendo que el libro quedaba con la frase NUEVA, y no quedaba; y la apertura siguiente, con
cualquier frase, borraba el `.retirado` —la única copia— y con la vieja abría `Ok` un libro vacío.
En las tres, la apertura que se paraba abría `Ok` un libro vacío, **sin ningún error**. En (b) y en (c)
**el libro, con sus saldos, se perdía entero**, y las dos lo decían igual de mal: el error del
recifrado nombraba la frase nueva, con la que el libro ya no abría, y con la de antes la apertura
abría `Ok` un libro vacío, sin ningún error. No crea saldo ni autoriza ningún gasto. **Alcance**: quien use la biblioteca con un
libro cifrado y dos aperturas a la vez, una de ellas un recifrado o una migración de la versión 1
—abrir un v1 es migrarlo—; el nodo y el cli abren en claro y nunca recifran, y el kit no depende de
la capa. **La ventana**: desde el §812. **Lo que lo cierra** (§816): un cerrojo sobre algo que no se
renombra, `<libro>.cerrojo`, un fichero con `flock` exclusivo que la capa crea —vacío; en Unix, con
0600— y no borra nunca. Lo tiene toda apertura desde antes de mirar los hermanos hasta que vuelve con
el libro abierto —y, si era v1, migrado—, y `recifrar`, desde antes de mirar los hermanos hasta el
final —tras el Argon2id de la frase nueva, antes de tocar el disco—; el que llega lo espera como al
de sled, diez veces cada 50 ms, y si no lo toma da `Io` con su nombre, sin tocar nada. No se crea
nada a través de un enlace: si no está, se crea con `create_new`, que no sigue un enlace en el último
nombre; uno que aparezca entre mirar y abrir se sigue, sin crear ni escribir nada en su destino
aunque le toma el `flock` un instante, y con el `flock` tomado se mira que el nombre siga siendo ese
fichero, por su dispositivo y su inodo, y se rehúsa si no lo es.
Quien lo tiene lo vuelve a mirar antes de lo que no se deshace —el `sled::open` de la apertura y el
compromiso del recifrado—, y si alguien lo borró o lo cambió, rehúsa con `Malformed` sin tocar nada.
La apertura crea antes el directorio del libro si no está, como lo creaba `sled::open`; `recifrar`,
no, ni el `.cerrojo` donde no hay libro ni hermanos. Por una ruta sin nombre —que acaba en `..`, como
`<libro>/blobs/..`, y puede nombrar el libro— no se abre ni se recifra: sin nombre no hay cerrojo. El
cerrojo no es `Send`: no cruza de hilo. Con el cerrojo tomado, la apertura no crea un libro donde hay
un `.retirado` o un `.recifrando` sin libro. Medido con las mismas pausas, en tests de la suite: las tres carreras —la apertura abre el
libro entero, el recifrado da el `Io` del cerrojo sin tocar nada, y después ninguna apertura crea un
libro vacío ni borra la única copia, y el saldo sigue—, dos recifrados a la vez, una apertura que
llega mientras otro recifra o migra, el cerrojo, que ningún paso renombra ni borra, la reentrada en
el hilo, que no se queda tras volver, un libro nuevo en un directorio que no existe, un cerrojo que
cambia mientras se toma, uno que se borra o se cambia mientras se tiene, una ruta sin nombre, un
recifrado sin libro, que no deja el cerrojo, y el cerrojo, que no cruza de hilo; cada pieza cae con
su mutante, y los de la carrera caen sobre `b6994c3`. Con dos procesos de verdad lo midió un revisor
sobre el primer arreglo del §816, fuera de la suite, y la sesión con sus mismos tests sobre el arreglo
de cada ronda, también el último: se excluyen en los dos sentidos, y uno muerto con SIGKILL con el
cerrojo tomado no deja el libro cerrado. Por qué un fichero hermano y no un `flock`
sobre el directorio padre lo dice la D-J del RFC-0001. **Residuo**:

- un proceso con el código de antes del §816 sobre el mismo libro a la vez no toma el cerrojo, y con
  él la carrera sigue;
- borrar o sustituir el `.cerrojo` a mano —restaurar una copia, un `rsync`— mientras alguien lo tiene
  deja entrar a otro, que crea o toma uno nuevo: sobre el primer arreglo del §816, un revisor midió que
  con una apertura parada antes de `sled::open` era la carrera (a) otra vez. Desde el arreglo, quien lo
  tenía lo nota antes de `sled::open` y antes del compromiso, y rehúsa (hay test); queda la ventana
  entre esa comprobación y la llamada que sigue, y fuera de Unix sólo se mira que siga siendo un
  fichero. Con un recifrado ya pasado su compromiso, el cerrojo de sled de su `.retirado` sigue
  excluyendo (medido por el revisor);
- quien abre por un enlace al directorio del libro toma el cerrojo de junto al enlace; recifrar y
  migrar por un enlace se rehúsan, y en el hueco de un recifrado, entre los pasos 5 y 6, `sled::open`
  por el enlace roto da «File exists» y no crea nada (medido por un revisor del arreglo);
- un libro cuyo directorio padre no se puede escribir y sin su `.cerrojo` ya no abre, hasta que se
  crea a mano, vacío y para quien lo abre; si el padre es de un sistema de ficheros de sólo lectura
  —un libro que es un punto de montaje dentro de uno de sólo lectura—, el `.cerrojo` que ya está se
  abre sólo para leer, porque ahí nadie puede cambiarlo, y si no está hay que crearlo antes, donde se
  pueda escribir (medido fuera de la suite, con el cli y un montaje de sólo lectura);
- un `.cerrojo` que quien abre no puede abrir en escritura —de otro usuario y con 0600, como el que
  deja `root` si abre con el cli, antes que el nodo, el libro del nodo que aún no tenía el suyo— deja el
  libro sin abrir: `Io` con su nombre, que dice el remedio: con el libro cerrado por todos, quien pueda
  se lo da (`chown`) o lo borra, y la apertura siguiente lo crea. Hasta el §816 se abría. Abrirlo sólo
  para leer no lo arreglaría: con 0600, otro usuario tampoco lo lee (medido fuera de la suite, sin
  `CAP_DAC_OVERRIDE` ni `CAP_DAC_READ_SEARCH`);
- por una ruta sin nombre —`.`, `..`, o una que acabe en `..`— ya no se abre ningún libro (`Malformed`);
  hasta el §816 se abría lo que nombrara;
- no está probado en Windows ni en NFS, donde el cliente emula `flock` con `fcntl`.

**Lo que la biblioteca no controla, y no se promete.** Borrar es borrar, no sobrescribir. No se
sobrescribe antes de borrar: lo decidió el autor (§812), por recomendación de la sesión —no se
puede medir aquí, no vale en un SSD, en un sistema copy-on-write ni con el journal, e invitaría a
prometer—. Se borra, y se dice lo que puede seguir guardando copias:

- **los bloques que el sistema de ficheros libera** al borrar el directorio viejo; y los mismos
  bloques de otros restos: el inodo del keystore v1 que la E3 reescribe con un `rename` (deducido,
  sin medir: en la máquina de la sesión no se podía leer el dispositivo), los segmentos que sled
  trunca, la instantánea anterior, que `export_snapshot` sustituye entera con un cambio de nombre
  (⚠️ §820: decía «que `export_snapshot` trunca en sitio (la entrada 120 del `BACKLOG.md`)»; la 120 la
  cierra el §820, y los bloques de la anterior se liberan igual. Una exportación muerta deja junto al
  destino `{instantánea}.exportando`, entero o a medias, con lo que llevaría el destino —sellado con
  la clave que tenía la capa, o en claro si no tenía clave; uno entero de antes de un `recifrar` abre
  con la frase vieja (deducido, sin medir), y uno a medias no se importa—; no se barre: la exportación
  siguiente a ese destino rehúsa nombrándolo, y se borra a mano. La nueva toma de la anterior, con
  privilegios, su dueño, y su grupo y sus bits de modo, pero no sus ACL POSIX ni sus demás atributos
  extendidos: con una ACL extendida, los bits de grupo del modo son su máscara, y la nueva se los da
  al grupo dueño, que lee así una instantánea —en claro, si la capa no tiene clave— que antes no
  leía), y los `{instantánea}.tmp-descifrada` que la importación escribía EN CLARO y borraba, entre
  `5d2833e` (28-07-2026) y `c87d7d6` (§774, 08-10-2026);
- **un `.tmp-descifrada` entero, en claro**, si un proceso murió a mitad de una importación en esa
  ventana: el §774 quitó el escritor y no barría. Desde el §812, por decisión del autor, la
  importación de esa misma instantánea lo barre: si junto a ella hay un fichero con ese nombre, lo
  borra (`la_importacion_barre_el_claro_que_dejaba_una_importacion_muerta`) y lo dice por la salida
  de errores. Es de la sesión, reversible, que si no puede borrarlo la importación falle con un error
  que lo nombra, sin importar nada, y que sólo borre un fichero regular: un directorio o un enlace
  con ese nombre no se tocan. ⚠️ §819: decía «Ni la negativa ni el aviso tienen test». Los dos lo
  tienen desde el §819 (`si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada`): la
  negativa, con el borrado mismo hecho fallar, y el aviso, leído en lo que guarda el hilo en los
  tests; que vaya a la salida de errores no lo ve el test: fuera de los tests es un `eprintln!` de
  una línea, sin test, y se declara; dejarlo así es de la sesión, reversible. El de una instantánea
  que no se vuelva a importar se busca junto a ella y se borra a mano; y borrarlo tampoco sobrescribe;
- **la capa importada** de una instantánea cifrada nace sin clave y, si se vuelve a exportar, sale
  en claro (la entrada 121 del `BACKLOG.md`; ⚠️ §821: ya no: desde el §821, por decisión del autor, la
  capa que devuelve `import_snapshot_with_key` conserva la clave de la instantánea, fijada a su
  cabecera, y se reexporta cifrada, con la misma frase; y por decisión de la sesión, reversible, la
  de una v1, en la versión 2, con sal nueva; la 121 la cierra el §821, y la 3.16 lo dice. Lo que
  exporta una capa sin clave, como la de `import_snapshot`, sigue en claro. Y la clave que conserva
  es la de la instantánea, no la del libro: una copia de antes de un `recifrar`, importada con la
  frase vieja y reexportada, sale cifrada con la frase vieja y con la cabecera de esa copia, no con
  la del libro recifrado, y no abre con la nueva (medido en la sesión, con un test que no queda en
  el árbol: el asiento del §821). Cada reexportación de una copia restaurada con la frase vieja es
  una copia NUEVA bajo esa frase, con lo que lleve entonces la capa importada, y quien cambió la
  frase porque se filtró la tiene que contar entre las copias que guardan la vieja);
- **el hardware y el sistema de ficheros**: un SSD (su FTL, el nivelado de desgaste, TRIM), los
  sistemas copy-on-write o de log (btrfs, ZFS, APFS, F2FS) y sus instantáneas, las de LVM, de una
  máquina virtual o de la nube, y el journal con `data=journal`;
- **las copias**: del directorio, las instantáneas exportadas antes y un enlace duro hecho fuera
  siguen abriendo con la frase vieja; las de un libro migrado llevan además el resto v1;
- **el keystore**: cambiar su frase (`load` y `save`) no cambia la clave de gasto, y una copia vieja
  abierta con su frase vieja da la misma clave;
- **la memoria**: swap, hibernación y volcados.

**Los dos nombres son reservados**: no hay marca que distinga un hermano que dejó el recifrado de un
directorio que alguien llamó así, y la apertura BORRA un `<libro>.recifrando` o un
`<libro>.retirado` de sled que encuentre junto al libro y que nadie tenga abierto, también una copia
de seguridad con ese nombre. Lo que no es de sled no lo toca. ⚠️ §816: y un tercero, `<libro>.cerrojo`,
el cerrojo del libro, que la capa no borra nunca; uno que no es un fichero —un directorio, un enlace—
hace rehusar la apertura.

El nodo abre su libro en claro (§3.3): no tiene frase que cambiar ni resto de la v1 que limpiar, y
cablear su cifrado queda fuera. Pero su apertura es la de la capa, y también resuelve los dos
nombres reservados junto a su libro, con el mismo borrado y la misma negativa: un `.recifrando` o un
`.retirado` de sled al lado del libro del nodo los resuelve al arrancar como cualquier apertura —los
borra, o, si falta el libro y está su `.retirado`, pone el `.recifrando` en su sitio—, y un estado
que el recifrado no deja le impide arrancar. ⚠️ §816: y toma el cerrojo del libro como cualquier
apertura: al arrancar, el nodo crea `<libro>.cerrojo` junto a su libro si no está —y, en su primer
arranque en una ruta nueva, el directorio del libro, como antes del §816—.

Se redactó en privado y se publica de una vez con el código, como pide la sección 5. El aviso es
esta sección y el asiento §812, sin GHSA, por decisión del autor: el kit publicado no está afectado
—no depende de la capa—, así que no lleva aviso ni release. Un marcador para distinguir los libros
ya migrados no se pone; eso es de la sesión, y el autor no lo ha decidido: sería un formato nuevo y
daría a entender que con eso basta; es reversible. `AUDITORIA.md` §812. ⚠️ §816: el arreglo de la
carrera se redactó en privado, como pide la sección 5, y se publica como el §812, por decisión del
autor: de una vez con el código, y el aviso es esta sección y el asiento §816, sin GHSA ni release,
porque el kit publicado no depende de la capa. El cerrojo elegido es de la sesión, y reversible
mientras el RFC-0001 siga PROPUESTO. `AUDITORIA.md` §816.

### 3.15 El recibo de recepción nombraba en ROJO a un operador honrado, y en PARADA el nodo seguía firmando — ⚠️ MEDIDO, cerrado en el nodo y en el kit 0.4.3 (§815)

Tres fallos de la misma familia: el recibo de recepción del RFC-0010, que existe para que un
operador que censura quede nombrado, nombraba también al que no censura, y la PARADA del §655, que
existe para que la memoria no firme lo que el disco no tiene, no paraba al que firma. El primero lo
dedujo, leyendo el orden en que el nodo anota y aplica, quien preparaba la decisión de la 19 del
`BACKLOG.md` (sin log de escritura anticipada); los otros dos salieron al diseñar y juzgar su
arreglo, y el tercero lo anotaba ya por lectura el borrador `doc/ecst/ECST.md` §8.2. Los tres se
midieron en privado, en un clon sin remoto, sobre `4d8017d` (el §813), con nodos de verdad que
firman, inyectando la muerte y el disco lleno con `strace`. El código de `crates/` no cambia entre
`4d8017d` y `ff9d4fc` (el §838; `git diff --stat`), así que lo medido es lo de hoy. Los registros
de esas medidas se quedan fuera del árbol, en los clones privados, y no se publican; el asiento §815
dice qué midió cada uno y con qué.

**F1 — una caída entre anotar y aplicar dejaba una hoja sin veredicto.** Desde el §569 el nodo
reservaba el `rx`, ANOTABA la hoja con `fsync` y sólo después la capa juzgaba. Medido: con el
proceso muerto entre anotar y aplicar —un aborto inyectado tras anotar, y un SIGKILL inyectado por
`strace`—, la hoja queda en el disco; al rearrancar, el registro y el contador están «a la par», sin
aviso; la primera cabeza firmada la acredita (`recepCount` 1) y `zkssl_recepPath` la sirve; la
operación no está en el libro. El titular recibió un 502 y ningún recibo, y aun así lo recuperó con
`zkssl_recepPath` y el kit (33 llamadas, 1 acierto). Si no reenvía, pasada la ventana —1.440
cabezas; 2.875 s en la medida— el sobre de completitud sale ROJO, «NO RESUELTA EN LA VENTANA»,
contra un operador que no mintió; si reenvía cuando el estado ya se movió, `StaleState`. Visto desde
fuera es el mismo estado firmado que el mentiroso del §602 fabrica restaurando el libro. Un pánico
de la capa o un fallo del almacén después de anotar dejaban lo mismo (deducido de leer el orden; en
la medida de F2, abajo, el `rx` del envío que paró el nodo quedó acreditado, `recepCount` 3, y su
operación no estaba en el libro al rearrancar).
**Ventana**: en `applySend` y `applyClaim` desde el §569, el §570 y el §573 (28-09-2026: la hoja, la
hoja firmada y el veredicto del kit); en `applyMany` y `zkssl_pledge`, desde el §611 (30-09-2026);
hasta el §815.

**F2 — en PARADA, el latido seguía firmando.** Desde el §655 un fallo del almacén pone el nodo en
PARADA, y el despacho deja de servir; pero `latido::latir` no miraba la PARADA. Medido con el disco
lleno (`ENOSPC` inyectado en el libro) durante un envío: la respuesta es la PARADA, el nodo rechaza
toda petición, y el latido firma cabezas desde la memoria, que lleva el envío que el disco no tiene:
cabezas del `seq` 9 con el disco en el `seq` 8, diez en 20 s en una corrida y cinco en 10 s en otra.
Tras rearrancar, el libro reabre en el `seq` 8, y en la segunda corrida el nodo vuelve a firmar un
`seq` 9, el de otra operación: dos cabezas firmadas a la misma altura con historias distintas contra
un operador honrado (deducido de que la operación es otra; la medida no comparó su `chainDigest`).
El testigo, `--auditar` y `--ausentes` no dieron ningún hallazgo. Y `zkssl_publishConsumo` devolvía
el fallo del almacén dentro de un resultado (`accepted: false`, causa `Store`), sin PARADA: medido
con el binario de antes del arreglo, el nodo siguió sirviendo y firmó 6 cabezas en 10 s.
**Ventana**: el nodo firma sus cabezas desde el §241-§243 (09-08-2026); hasta el §655 (01-10-2026)
un fallo del almacén ni siquiera paraba el despacho, y desde él la 3.9 lo daba por cerrado;
`zkssl_publishConsumo`, desde el §417 (06-09-2026). Hasta el §815.

**F3 — un rechazo por prueba inválida no tenía resolución.** Un tercero **sin cuenta**, con las
raíces de `zkssl_epochHead` y el límite de `zkssl_params` —lecturas públicas— y 32 bytes `0xab` por
prueba, manda un envío. El nodo lo rechaza con `VerificationFailed` («prueba mal formada»), le da su
recibo en `error.data.recepcion` y deja la hoja bajo la `recepRoot` firmada; el cobro, igual. El
operador honrado no tenía ninguna resolución que el kit aceptara: el sobre de rechazo sale ROJO, «la
causa VerificationFailed no la prueba este mando», y la declarada también, «no es de las que el
RFC-0007 dejo sin prueba portable», aunque la D-I del RFC-0007 las declara así y la D-G del RFC-0010
las manda al cuarto estado. Pasada la ventana —una cabeza vigente de índice 1444, en 48 minutos—, el
sobre sin resolución sale «NO RESUELTA EN LA VENTANA»: el operador queda nombrado por la basura de
un tercero. Sólo mintiendo la causa —declarando `PendingTreeExhausted`— bajaba a 3. Medido de punta
a punta. Un cliente honrado de otro punto de la historia cuya prueba no verifica (RFC-0021, hecho 1)
dejaba lo mismo (deducido). `ProofFailed` va con ella porque la D-I la declara igual; por lectura,
la capa sólo la emite al probar, no al juzgar lo que llega. **Ventana**: en el árbol desde el §573
(28-09-2026), y en el lote desde el §612; en las releases del kit que leen el sobre de completitud,
la 0.3.0, la 0.4.0, la 0.4.1 y la 0.4.2, que llevan la misma lista de cuatro causas (leído en sus
commits; medido con la 0.4.2); la 0.2.0 no lo lee.

**Alcance.** Ninguno de los tres crea saldo, gasta sin autorización ni hace que el kit acepte una
transición inválida: no son fallos de solidez en el sentido de la sección 5. Lo que rompían es la
evidencia contra el operador. F1 y F3 daban un ROJO firmado, «NO RESUELTA EN LA VENTANA», contra un
operador que no censuró: F1 con una caída en el instante justo, y F3 a voluntad de cualquiera, sin
cuenta y con lecturas públicas, por cada envío o cobro basura que mande. F2 firmaba cabezas de un
estado que el disco no tiene, y tras rearrancar el nodo firmaba otra a la misma altura con otra
operación: dos cabezas firmadas incompatibles para un mismo `seq`, de un operador honrado
(deducido: la medida no comparó sus `chainDigest`). F2 alcanza al nodo que firma sus cabezas; el
ROJO de F1 y F3 lo da todo kit que lea el sobre de completitud del §573.

**Lo que lo cierra** (§815), por decisión del autor:

- **F1, en el nodo: juzgar primero y anotar después** (RFC-0010, la D-E enmendada). En los cuatro
  brazos con recibo, el nodo reserva el `rx` y fija la era al recibir, la capa juzga, y la hoja se
  anota con `fsync` tras el veredicto y antes de responder. Una evaluación cortada deja un hueco,
  que no promete nada; con `--ledger`, la hoja de una operación aplicada nace tras el `flush` de su
  asiento (sin él, no: abajo, el residuo). Un fallo del almacén no deja hoja y para el nodo. Si el
  registro falla tras el veredicto, la petición recibe su veredicto sin recibo y el nodo para: se
  revierte la decisión (a) del §569, con su razón en el RFC. Una marca de fallo en vuelo, escrita
  con el candado del estado tomado, hace que la petición que esperaba ese candado no se evalúe (la
  carrera, medida con hilos reales por una de las revisiones del §815, con su test en el sello). Y una
  línea de traza con el método, el `rx` y el `hashPrueba` —sin los parámetros— sale antes de que la
  capa juzgue.
- **F2, en el nodo**: `latir` no compone ni firma con la PARADA puesta, ni con la marca de fallo en
  vuelo, y no gasta índice XMSS; `zkssl_publishConsumo` devuelve el fallo del almacén como error, y
  el nodo para.
- **F3, en el kit 0.4.3**: `ProofFailed` y `VerificationFailed` se declaran —salida 3, «DECLARADA,
  NO PROBADA»— en la vía directa y en el lote. Su sobre de rechazo sigue sin probarse. Tres vectores
  nuevos de `spec/vectors/completitud/` lo fijan, y ningún vector publicado se mueve. `doc/KIT.md`
  §0 dice qué veredictos cambian frente a la 0.4.2 publicada (RFC-0021, R6 f).

**Medido después**, con el nodo y el kit del §815 y, como falsadores, los de antes:

- con un SIGKILL tras el juicio y antes de anotar, el `rx` queda hueco, el arranque lo avisa,
  `zkssl_recepPath` lo da por hueco, y el pago está en el libro, con su acuse;
- con un SIGKILL tras el `fsync` de la hoja, la hoja está y el pago también, y el sobre resuelto con
  su acuse sale VERDE; con el nodo de antes y la misma inyección, la hoja está, el pago no, y el
  mismo sobre sale ROJO;
- en PARADA por un envío con el disco lleno, el diario no gana ninguna cabeza en 10 s y el índice de
  firma no se mueve, y tras rearrancar ninguna altura tiene dos cabezas firmadas distintas; en
  PARADA por un consumo, ninguna cabeza en 10 s y el índice quieto, frente a las 6 cabezas del nodo
  de antes (sin rearrancar: lo de las dos cabezas a la misma altura sólo se midió con el envío);
- los sobres de la medida de F3, con el kit 0.4.3: la declarada con `VerificationFailed` sale 3, y
  el sobre de rechazo, 1, como con la 0.4.2: sigue sin probarse. Sobre los 500 vectores de
  `spec/vectors/`, frente al kit del árbol sin el arreglo, cambian sólo los tres nuevos.

**Residuo**:

- **`ProofFailed` y `VerificationFailed` siguen sin probarse.** Que la prueba no verificara es la
  palabra del nodo hasta que el kit pueda re-verificarla contra lo que la hoja firma: con el
  `hashPrueba` y el `digest_pi` bajo la firma, quien tenga la prueba la presenta y el rechazo se
  prueba o se refuta. Es del tren `zkssl/0.5` (RFC-0019 E3), y así lo decidió el autor.
- **La declarada no la ata nada.** Medido en privado sobre `4d8017d`: el kit da 3 a una declarada
  con una causa de la lista que no viene a cuento —`NotTheAccountHolder` sobre un envío— y hasta sin
  cabeza vigente; lo único que la ata al recibo es el `hashPrueba`, que el operador conoce. Por eso
  añadir dos causas no abre una puerta: cualquier ROJO ya bajaba a 3 declarando una de las cuatro.
  La salida 3 es la palabra del operador, y nada más (RFC-0010, D-G). `ProofFailed`, además, no la
  produce hoy ningún brazo con recibo —la capa sólo la da al probar; leído en el código—, así que
  una declarada con ella es siempre esa palabra. Y en el lote el kit no cruza la causa declarada
  con la composición: `lote-declarada` (publicado desde el §614) declara `NotTheIssuer` sobre una
  composición que repite la cuenta, que la refutaría sola. La 129 del `BACKLOG.md`.
- **El rechazo que nadie conserva** (deducido, sin medir). El nodo no guarda el `data` de un
  rechazo. Si muere entre el `fsync` de la hoja de un rechazo y la respuesta, si la respuesta se
  pierde o si el titular la calla, el operador honrado no tiene el sobre que exhibir, y una causa
  con prueba no se declara: pasada la ventana, ROJO. Es anterior al §815, y el §815 no lo abre ni lo
  cierra. La 129.
- **Sin `--ledger`, la hoja sin su operación** (deducido de leer el código, sin medir; la A8 de
  dos de las revisiones que juzgaron el orden). Un nodo que firma sólo necesita `--diario` junto a
  `--clave` (§285); sin `--ledger` el libro vive en memoria y su `commit` no escribe nada, mientras
  el contador y el registro de recepción van siempre a disco. Tras reiniciar, las hojas de las
  operaciones aplicadas siguen bajo una `recepRoot` firmada y sus operaciones no están en el libro:
  el estado de F1, y el mismo ROJO al expirar la ventana. Es anterior al §815, que no lo empeora;
  exigir `--ledger` a quien firma queda fuera de este sello.
- **Más huecos.** Toda evaluación cortada deja ahora un hueco, que el arranque declara. El titular
  sin respuesta no sabe si su operación se aplicó: **busca su `hashPrueba` entre los `proofDigest`
  de `zkssl_logEntries` antes de volver a pagar** (`spec/RPC.md`). En `applyMany` y `zkssl_pledge`
  la PARADA de un fallo del almacén no lleva su `rx` en el mensaje, al revés que en `applySend` y
  `applyClaim` (leído en el código).
- **La traza cuesta bytes.** La línea «recepcion: la capa juzga» sale por cada recepción juzgada,
  también por la basura de un tercero sin cuenta; con la hoja y la línea de error del despacho,
  suma log por petición y acerca un disco lleno, que lleva a la PARADA (deducido, sin medir; el
  coste por petición ya existía, §253, y el destino del log es del despliegue).
- **Sin medir**: la latencia de `applySend` antes y después (las mismas dos escrituras, en otro
  orden: deducido); la muerte entre el `fsync` de la hoja de un rechazo y la respuesta; el testigo
  frente a un nodo en PARADA; y la ventana de 1.440 cabezas de punta a punta con el nodo del §815.

**Lo que conviene hacer.** Un ROJO «NO RESUELTA EN LA VENTANA» obtenido con la 0.3.0, la 0.4.0, la
0.4.1 o la 0.4.2 sobre un recibo que el nodo rechazó por `VerificationFailed` o `ProofFailed` no
acusa a nadie: se vuelve a juzgar con la 0.4.3 y la resolución `declarada`, que sale 3. Un ROJO de
esa clase sobre un recibo de la ventana de F1 —en `applySend` y `applyClaim` desde el §569, en
`applyMany` y `zkssl_pledge` desde el §611, hasta el §815— puede ser una caída honrada entre anotar y
aplicar, que desde fuera no se distingue del mentiroso del §602: por sí solo no prueba censura.
Quien firme cabezas debería arrancar el nodo con `--ledger`, que persiste el libro.

Se redactó en privado y se publica de una vez con su arreglo, como pide la sección 5 y como la 3.14.
A diferencia de la 3.13 y la 3.14, el kit publicado SÍ está afectado (F3): con este sello salen,
como piden la sección 5 y `spec/rfc/PROCESO.md` §«Fallos de seguridad», la release
`arqueo-verify-v0.4.3` y su aviso de seguridad, que nombra las afectadas —la 0.3.0, la 0.4.0, la
0.4.1 y la 0.4.2— y la primera que lo arregla, la 0.4.3. Los publica el autor: ~~hasta el asiento que
registre la release, la vigente sigue siendo la 0.4.2~~ (`doc/KIT.md` §0). ⚠️ §818: los publicó
después del sello: la release `arqueo-verify-v0.4.3`, sobre `5fd5212`, y el aviso `GHSA-6p4m-qg2w-8r6j`, de
severidad media (CVSS 4,3) (<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/security/advisories/GHSA-6p4m-qg2w-8r6j>), con esas afectadas y esa arreglada; el §818 registra la release
con su huella, y la vigente es la 0.4.3. F1 y F2 son del nodo, que
no tiene release: su aviso es esta sección y el asiento. `AUDITORIA.md` §815.

### 3.16 La importación con clave aceptaba una instantánea en claro — ⚠️ MEDIDO, cerrado en la biblioteca (§817)

`SovereignLayer::import_snapshot_with_key`, que importa una instantánea con la frase del libro,
aceptaba también una con la marca `0x00`, en claro: la importaba sin error ni aviso, sin mirar la
clave. Lo anotó, leyendo el código, un revisor de los sellos de la instantánea que preparaba la
sesión, y se midió en privado, en un clon sin remoto, sobre el §816: una capa cifrada exporta su
instantánea; en su sitio se pone una en claro, hecha con la biblioteca, con otro saldo; y con la
frase buena la importación da Ok, con el saldo de quien la puso. Lo mide el test del §817, que con
el código de antes falla en eso mismo.

**Alcance.** Quien pueda escribir donde se guarda la copia cifrada —un disco de copias, un almacén
compartido— y quien la restaure con su frase: recibe Ok y una capa con las cuentas, los saldos, las
congeladas y las retiradas que eligió quien la puso, y también con sus custodios y su gobernanza
—las raíces de los dos conjuntos, que deciden quién emite, congela, recupera y cambia los
custodios— y sus topes —el límite regulatorio, el suministro máximo y el máximo de cuentas—, y nada
lo dice. La importación toma esos conjuntos y esos topes del fichero sin compararlos con nada
(`crates/zk-ssl/src/snapshot.rs:391-392` y `:413-415`). El cifrado autenticado de la instantánea
(§702) no protegía lo que se restaura, porque si se autentica o no lo decidía la marca, que pone el
fichero. No alcanza al nodo, al cli ni al sdk, que no importan instantáneas con clave (la función
sólo la llaman los tests de la capa; `git grep`), ni al kit, que no depende de la capa. La ventana
va desde `5d2833e` (28-07-2026), donde nace la función, hasta el §817. Lo dejaban falso dos
comentarios de la cabecera de `snapshot.rs` —la instantánea cifrada, y la cola de retiradas que
sólo ata el sellado— y la 3.13 («Solo la cifrada las protege»), que llevan marca.

**El arreglo.** Con clave, la marca `0x00` se rehúsa, con un `Malformed` cuyo texto dice que la
instantánea está en claro y que, si se esperaba una cifrada, pudo ponerla quien escribe donde se
guarda: `import_snapshot`, que no recibe clave, la importaría sin autenticar nada. Falla cerrado.
Lo ata `con_clave_una_instantanea_en_claro_no_se_importa`, al final de `crates/zk-ssl/src/tests.rs`
(⚠️ §819: ya no es el último: detrás va el del §819, en la 3.14; ⚠️ §820: y detrás de ése, los tres del §820, también en la 3.14; ⚠️ §821: y detrás de ésos, los dos del §821, abajo, en «Lo que no hace»; ⚠️ §828: y detrás de ésos, los doce del §828, en la 3.17): con la frase, con otra frase y con una
clave ya fijada a su KDF, una en claro no se importa; el mismo cuerpo tras una marca que no es de
cifrado, tampoco; ni una v8 en claro; y sin clave, sí.

**Lo que no hace.** `import_snapshot` sigue importando una en claro: es la vía de una capa sin clave,
y quien la llama tiene que saber de dónde sale. La importación con clave no ata la instantánea a un
libro ni a un momento: una cifrada con la misma frase se importa aunque sea de otro libro (medido
fuera del árbol, en la revisión del §817: la de otro libro, puesta en el sitio de la buena, da Ok
con sus saldos) o de antes (deducido: lo que el cifrado autentica es la clave que la frase deriva
con la sal de su cabecera, no cuál de las instantáneas selladas con ella es). Y la 121 del
`BACKLOG.md`, la capa importada que nace sin clave y se reexporta en claro, sigue abierta; por ella,
una capa importada con clave y vuelta a exportar sale en claro, y esa copia, que no se distingue
de una puesta por otro, ya no se importa con clave: da el error nuevo. Falla cerrado, pero quien
encadene importar, exportar e importar verá un error que antes no veía. ⚠️ §821: la 121 la cierra
el §821, por decisión del autor: la capa importada con clave conserva la clave de la instantánea,
fijada a su cabecera, y se reexporta cifrada, con la misma frase; la reexportada se vuelve a
importar con la frase, y quien encadena importar, exportar e importar ya no ve ese error (`una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera`
y `una_capa_importada_de_una_v1_se_reexporta_en_la_version_2`, al final de `crates/zk-ssl/src/tests.rs`; ⚠️ §828: ya no son los últimos: detrás van los doce del §828, en la 3.17).
Y dos decisiones de la sesión, reversibles: la de una v1 se reexporta en la versión 2, con sal
nueva; y una v1 ya no se importa con una clave ya fijada (`LedgerKey::derivar`), que no lleva la
frase, porque la sal nueva pide derivarla con la frase: quien la importe así verá un error que antes
no veía, que lo dice y nombra `LedgerKey::from_passphrase`. Falla cerrado. Lo que exporta una capa
sin clave, como la de `import_snapshot`, sigue en claro, y esa copia sigue sin importarse con clave.

Se redactó en privado y se publica de una vez con su arreglo, como pide la sección 5, y como el §812
y el §816, por decisión del autor: el aviso es esta sección y el asiento §817, sin GHSA ni release,
porque el kit publicado no depende de la capa. `AUDITORIA.md` §817.

### 3.17 El recifrado y la migración v1→v2 dejaban el libro del usuario y del grupo de quien los corría — ⚠️ MEDIDO, cerrado en la biblioteca (§828)

El recifrado de la E4 (`SovereignLayer::recifrar`, §812, `crates/zk-ssl/src/persistence.rs:1318`) y
la migración del cifrado en reposo de la versión 1 a la 2 —desde el §812, la misma pieza, que corre
en toda apertura con clave de un libro v1 (`migrar_a_kdf_v2`, :1050)— llaman a `recifrar_en_hermano`
(en :1353 y en :1064), que escribe el libro en un directorio hermano, R (`<libro>.recifrando`), y lo
pone en su sitio.
R nacía del usuario y del grupo del proceso —el directorio, en `nace.create(&r)` (:1421); `conf`,
`db`, los `snap.*` y `blobs/`, al abrir sled sobre R (:1501)—, o del grupo del padre si el padre tiene
setgid. El paso 4 le daba sólo los modos del libro (`copiar_modos`, :2143, y el del directorio,
:1598-1599), y R tomaba el sitio del libro en el paso 6 (:1456) o, tras una muerte entre los pasos 5
y 6, en la apertura siguiente (`resolver_restos`, :1702). Lo vio la sesión leyendo el código, y se
midió en privado, en un clon sin remoto, sobre el §821, con `setpriv`:

- **root** que recifra o migra un libro de 12345:12346 lo deja de 0:0, con los mismos modos, y su
  dueño ya no lo abre: «IO error: Permission denied (os error 13)» en 5 de 5 libros, y de esos cinco
  lee 0 de sus 25 entradas, que antes leía todas. Tras una muerte en el paso 5 tampoco termina el
  recifrado («….recifrando: Permission denied»): el libro se queda en su `.retirado` y su
  `.recifrando` hasta que actúe root. El contenido no cambia: root lo abre, con su saldo;
- **el dueño sin privilegios con otro grupo principal** —12345, de grupo principal 12345 y con 12346
  de suplementario— deja un libro de 12345:12346 en 12345:12345, con los mismos modos: con 0750 y
  0640, el grupo del libro pierde la lectura (12347:12346, 0 de 10 entradas) y el grupo principal del
  dueño la gana (12348:12345, 10 de 10). Ser del grupo 12346 no lo evita. Con un padre setgid, el
  grupo se conserva.

Quien gana la lectura lee lo sellado y lo que va en claro —la cabecera del KDF, con su sal y su
coste—, y con eso puede probar frases fuera de línea contra Argon2id, la amenaza que ya declara el
RFC-0001; la clave no cambia de manos, y root ya lo leía todo. **Alcance**: quien integra la
biblioteca y recifra, o abre con clave un libro v1, como otro usuario —root o `sudo`, en un
mantenimiento— o con otro grupo principal —un servicio—. Ningún binario del repositorio llama a
`recifrar` ni abre con clave: el nodo y el cli abren sin clave (`SovereignLayer::open`,
`crates/zk-ssl-node/src/main.rs:2293` y `crates/zk-ssl-cli/src/sandbox.rs:57`), y esa apertura sólo
encuentra un R si antes murió un recifrado de la biblioteca. El kit no depende de la capa. **La
ventana**: desde `b6994c3` (§812, 14-10-2026), para el recifrado y para la migración; antes, la
migración era un lote sobre el mismo sled y no movía los ficheros (deducido del código de
`b6994c3^`, sin compilarlo). Lo dejaban falso la cabecera de `copiar_modos`, los pasos 1 y 4 de la E4
en el RFC-0001 y la fila de R de su tabla de nombres reservados, que llevan marca.

**El arreglo** (§828). Lo que decidió el autor es una elección (21-10-2026, en el chat): a una
pregunta de opción múltiple de la sesión, eligió la opción «Rehusar si no es el dueño
(Recomendado)», y no la otra, «Copiar dueño y grupo como root». La descripción de la opción que
eligió, escrita por la sesión, decía: «recifrar y la migración v1→v2 fallan cerrado si quien los
corre no es el dueño del libro (por ejemplo, root sobre el libro de un servicio) y el error dice que
se haga como su dueño. Si es el dueño, conserva el grupo del libro si es uno de los suyos; si no,
quita los bits de grupo y de otros, como la 120. Nunca hace chown como root. Menos código y nada de
superficie nueva.» Todo lo demás es de la sesión, y reversible: dónde y cómo, y cómo lee esa
descripción. Dos frases de la descripción las lee así. «Nunca hace chown como root» no es exacto del
código: nunca se cambia el usuario de nada, ni el grupo ni los modos de lo que no creó quien
recifra; pero root, cuando es el dueño del libro, sí cambia el grupo —`fchown(fd, None, Some(gid))`—
de lo que él mismo creó dentro de un R suyo con 0700, y de R (el segundo punto de abajo). Y «si es
uno de los suyos», como el grupo que el `fchown` le deja poner: a root, cualquiera, también uno que
no es de sus grupos, como el 12346 del segundo test; y si el `fchown` falla, por lo que sea, quita
los bits. Que `recifrar` compare, además, antes de tomar el cerrojo del libro y de abrirlo, también es
de la sesión, y reversible: se hizo después de medir lo que dejaba la apertura que precede a la guarda
del paso 1 (en «Lo que no hace»). Así lo hace:

- **el libro de otro dueño no se recifra ni se migra.** En Unix se compara el usuario de quien recifra
  o migra con el del directorio del libro y con el de su `db`, su `conf` y su `blobs/`: los que sled
  abre para escribir, y los que el paso 6 dejaría de quien recifra. `recifrar` lo compara antes de
  tomar el cerrojo del libro y de abrirlo (`quien_recifra_es_el_dueno`, al final de `persistence.rs`);
  el usuario de quien recifra lo aprende sin `libc`, del dueño de una sonda que crea y borra en el
  momento junto al libro —un fichero vacío `<libro>.sonda.<pid>.<n>`, con `create_new`, que no sigue
  un enlace ni abre lo que ya lleve ese nombre, y 0600, mirado por su descriptor—, que es el usuario
  con que el proceso crea en ese directorio, el que tendría R (por qué ahí, en «Lo que no hace»);
  junto a lo que no es de sled no la crea —lo mira antes, sin escribir—, y si no la puede borrar,
  rehúsa nombrándola (las dos cosas, con test: el undécimo); y si no lo puede saber —la sonda no se
  puede crear, o el libro no está, no es de sled o no se puede mirar—, no rehúsa por eso, y sigue (con
  test, con una sonda que no cabe en su nombre y con su nombre ya puesto —un enlace simbólico a un
  fichero de fuera que no está, uno a uno que está y uno duro—, que no se sigue: el duodécimo; con lo
  que no es de sled, el undécimo; con el libro que no está o no se puede mirar, deducido). Y en cuanto
  nace R —el paso 1— se compara lo mismo con su dueño, el usuario del proceso, que lo acaba de crear:
  en la migración es la única comparación, porque que el libro es v1 sólo se sabe al abrirlo; en el
  recifrado, el respaldo. Si alguno no es el mismo, se rehúsa con `Malformed` —y en el paso 1, R,
  vacío, se borra—: «el libro es del usuario 12345, y lo intenta recifrar el usuario 0: asi quedaria
  del usuario 0, y su dueno no lo abriria; recifralo como su dueno. No se copia, no se sella ni se
  renombra nada», y en la migración, «lo intenta migrar a la version 2 -abrirlo con su frase lo
  migra-» y «abrelo con su frase como su dueno, que lo migra». Un libro que no es de un solo dueño —el
  directorio de un `install -d -o root -g svc -m 2770` con los ficheros del servicio— no lo recifra ni
  lo migra ninguno de los dos, y el error nombra lo que es de otro: «…/db: es del usuario 12345, y el
  directorio del libro del usuario 0: el libro no es de un solo dueno, y lo intenta recifrar el
  usuario 0, que lo dejaria todo suyo; que sea todo de su dueno, y recifralo como su dueno». Un
  borrador anterior de este arreglo comparaba sólo el directorio, y con ese libro root recifraba y
  migraba, y `conf`, `db` y `blobs/` quedaban de root: lo midió una revisión, y el servicio ya no lo
  abría («Permission denied (os error 13)»). Los `snap.*` no se comparan: la apertura de root deja uno
  suyo en el libro de otro (abajo, en «Lo que no hace»), y rehusar por él sería de más (deducido). La
  negativa del recifrado llega antes de crear, abrir o escribir nada del libro o de su cerrojo —si la
  comparación de antes lo sabe; si no pudo crear su sonda, en el paso 1, después de tomar el cerrojo,
  y de crearlo si no estaba, y de abrir el libro—, y un libro v1 que se recifra no llega a migrarse;
  la de la migración, antes de copiar, sellar o renombrar nada —lo que deja antes la apertura de sled,
  en el primer punto de «Lo que no hace»—. Ninguna deja `.recifrando` ni `.retirado`, ni cambia el
  dueño, el grupo ni el modo de lo que había. La apertura con clave de un libro v1 de otro falla
  entera: no se abre sin migrar, y falla cerrado. R tiene que ser, además, lo que nació: un
  directorio, vacío; su dispositivo y su inodo se guardan, y el paso 4 los exige;
- **si es el dueño, R y cada fichero de R toman el grupo de su homónimo en el libro** —el que no lo
  tiene, un `snap.*` o un blob, que llevan su número, el de `db`; el directorio, el del libro—, con
  la regla de la exportación de la 120: `fchown` con el dueño en `None`, que el dueño de un fichero
  puede poner si es uno de sus grupos, y root, cualquiera, por un descriptor abierto sin seguir un
  enlace —el tipo y el inodo, antes y después de abrir, y en Linux x86 y arm, además, `O_NOFOLLOW`—;
  si el grupo no se puede poner, la entrada se queda sin los bits de grupo ni los de otros
  (`& 0o700`); y después, los modos del homónimo: también el setuid, que el `chown` de un fichero quita
  siempre, y el setgid, que quita si el fichero tiene el bit x del grupo (medido en la sesión, en
  Linux 6.18: un `fchown` de root deja 2640 en 2640 y lleva 2650 a 0650 y 4640 a 0640). El
  directorio, el último, después de sincronizar R entero. Nunca se cambia el usuario de nada: lo que
  el paso 6 pone en el sitio del libro es de quien recifra, y la guarda exige que sea el dueño del
  directorio del libro y de su `db`, su `conf` y su `blobs/`. Pero root, cuando es el dueño, sí
  cambia el grupo, como cualquier dueño: con `fchown(fd, None, Some(gid))`, el de lo que él mismo
  creó dentro del R que nació, suyo y con 0700, y el de R; a lo que no creó no le da grupo ni modos:
  rehúsa, y el recifrado que rehúsa borra R con lo que lleve, como desde el §812 (con lo que no
  alcanza esa guarda, en «Lo que no hace»). Es la lectura que hace la sesión de «Nunca hace chown
  como root», arriba. Es de la sesión, y reversible, rehusar antes del compromiso —se borra R y el
  libro sigue como estaba— si en R hay un enlace o algo que no es un fichero ni un directorio, un
  fichero con otro nombre (un enlace duro), una entrada de otro dueño o un directorio dentro de
  `blobs/`, o si R ya no es el directorio que nació —otro dispositivo u otro inodo—, del dueño del
  libro y con 0700;
- **la apertura no termina ni borra un `.recifrando` de otro dueño** —el que dejaba el código de antes
  si root recifraba o migraba y moría a mitad, o uno que puso alguien—: si su dueño no es el del libro
  —el del directorio y el de su `db`, su `conf` y su `blobs/`, o los de su `.retirado` si falta el
  libro—, o el `db`, el `conf` o el `blobs/` del `.recifrando` no son de su dueño, rehúsa con
  `Malformed` sin tocar nada, y dice de quién es cada uno y qué hacer. No se pierde el libro: con el
  libro al lado, está entero con la frase de antes; con su `.retirado`, están el `.retirado`, con la
  de antes, y el `.recifrando`, con la nueva. Rehusar también con el libro al lado, y no borrar ese
  `.recifrando`, es de la sesión, y reversible: falla cerrado. El `.recifrando` del dueño que la
  apertura termina tras una muerte entre los pasos 5 y 6 ya tomó en el paso 4, que va antes del 5, su
  grupo y sus modos, y los conserva; y uno del mismo dueño y de otro grupo —el que dejaba el código de
  antes, recifrado por su dueño con otro grupo principal— lo termina como antes, con su grupo;
- fuera de Unix, como antes: sin dueños que comparar ni grupos que poner.

Lo atan doce tests, al final de `crates/zk-ssl/src/tests.rs`. Seis piden root y, sin él, se saltan
enteros: el primero, el segundo, el quinto, el sexto, el séptimo y el décimo de los de abajo; el
cuarto corre sin root salvo su (c); el undécimo, salvo su (a) con otro dueño y su (d); el duodécimo,
salvo su (b) y su (d); y el tercero, el octavo y el noveno no lo piden. El (a) y el (b) del duodécimo
se saltan también, con o sin root, si el pid del proceso tiene menos de tres cifras: entonces no hay
nombre de libro en el que quepa su `.recifrando` y no quepa siempre la sonda (abajo); su (c) y su (d)
no dependen del pid. Lo que se salta lo dice por la salida
de errores, que libtest sólo muestra con `--nocapture` o `--show-output`: en la salida de siempre
salen «ok» (medido como 12345:12345 con `setpriv`, en el clon privado, con el binario del último
borrador: doce «ok» sin ninguna línea «sin medir», y once con `--nocapture`). Los doce:
`el_libro_de_otro_dueno_no_se_recifra_ni_se_migra_y_no_se_toca` —un libro v2 y uno v1 de
12345:12346, asentados antes (abiertos y cerrados tras su última escritura, para que la apertura de
root no escriba en ellos un `snap.*` suyo) y con su cerrojo: el recifrado de los dos rehúsa antes de
la pieza de la E4, que empieza una sola vez, la de la migración; la apertura con clave del v1 rehúsa
en su paso 0 o 1, con un error inyectado al acabar el paso 1 que la guarda tiene que adelantar; los
tres, con el texto que nombra a los dos usuarios y sin llegar al paso que da el grupo, y el libro queda
dueño a dueño como estaba—;
`el_libro_recifrado_o_migrado_por_su_dueno_conserva_su_grupo_y_sus_modos` —un libro de 0:12346, con
modos puestos a mano (el directorio 2750 y `conf` 4640): al acabar el paso 4, cada entrada de la
copia es del grupo 12346 y lleva el modo de su homónimo —al menos una, un `snap.*`, sin homónimo—, y
lo conserva en el libro—;
`sin_poder_poner_el_grupo_la_copia_se_queda_sin_bits_de_grupo_ni_de_otros` —con el gancho
`FALLAR_DUENO` de la 120, `& 0o700`—;
`un_enlace_o_lo_ajeno_en_la_copia_no_recibe_el_grupo_del_libro` —un enlace, un enlace duro, una
entrada de otro dueño y la copia abierta a 0755, puestos en R al empezar el paso 4: rehúsa, y el
destino del enlace no cambia—;
`la_apertura_no_termina_ni_borra_un_recifrando_de_otro_dueno`;
`el_libro_que_no_es_de_un_solo_dueno_no_se_recifra_ni_se_migra` —el directorio en 0:12346 con 2770,
y `db`, `conf` o `blobs/` de 12345: el recifrado rehúsa —que sea antes del cerrojo no lo ve este
test, porque el paso 1 da el mismo texto: lo ve el décimo—, y la migración de cada uno
en el paso 1, con el error inyectado, nombrándolo, y el libro queda como estaba; y la apertura no
borra un `.recifrando` de root junto a un libro con su `db` de otro, ni uno con su propio `db` de
otro—;
`el_recifrando_de_su_dueno_que_termina_la_apertura_conserva_su_grupo_y_sus_modos` —tras una muerte
en el paso 5, y con otro grupo en el `.recifrando`—;
`lo_que_lleva_blobs_en_la_copia_toma_lo_de_db_y_un_directorio_hace_rehusar` —el libro no tiene
blobs, y se pone en `blobs/` de la copia un fichero, que toma el grupo y el modo de `db`, y un
directorio, que hace rehusar—;
`la_copia_cambiada_por_otro_directorio_no_recibe_el_grupo_del_libro` —R cambiado por otro
directorio del proceso, con 0700 y un fichero dentro, entre crearlo y compararlo y al empezar el
paso 4: rehúsa, y el fichero sigue con su modo—;
`el_libro_sin_asentar_de_otro_dueno_rehusa_antes_del_cerrojo_y_no_recibe_nada` —en un directorio
de 12345:12346 con 0700, cuatro libros sin asentar y sin cerrojo, uno de 12345:12346 y tres de
0:12346 con su `db`, su `conf` o su `blobs/` de 12345: el test se corre a sí mismo en otro proceso,
con la umask 0077, y root los recifra: rehúsan con su texto, y el directorio y los cuatro libros
quedan entrada a entrada como estaban, también los bytes de `db` —ni un `snap.*`, ni un cerrojo, ni
la sonda, ni un hermano—; y su dueño, con `setpriv`, abre el suyo después, con su suministro—;
`lo_que_no_es_un_libro_no_recibe_la_sonda_y_la_que_no_se_borra_hace_rehusar` —recifrar un directorio
que no es de sled rehúsa como antes, sin crear la sonda y sin cambiar nada del directorio que lo
lleva, ni su fecha de modificación, también como root con el directorio de 12345:12346; con la sonda
cambiada por un directorio, que no se borra, rehúsa una sola vez, nombrándola, sin tomar el cerrojo
ni tocar el libro, y quitado ese directorio se recifra y la sonda no se queda; y como root, con el
padre `chattr +a` y el libro de 12345:12346, rehúsa nombrándola, y lo único nuevo en el padre es esa
sonda, vacía, de 0:0 y con 0600—;
y `la_sonda_que_no_se_puede_crear_no_hace_rehusar_y_el_paso_1_queda_de_respaldo` —la sonda que no se
crea, de dos maneras. Con el nombre más largo en el que cabe el `.recifrando` del libro, medido en el
directorio del test, en el que no cabe la sonda si su sufijo, `.sonda.<pid>.<n>`, es más largo que
`.recifrando`: con un pid de tres cifras o más; con uno de una o de dos no hay un nombre en el que
quepa `.recifrando` y no quepa siempre la sonda —la más corta, `.sonda.<pid>.0`, lleva diez u once
caracteres, y `.recifrando`, once—, y esa parte se salta y lo dice (medido con `unshare --pid`, con el
pid 1). Y con su nombre ya puesto, que no depende del pid: el test se corre a sí mismo en otro
proceso, en el que sólo corre él y la cuenta de sondas empieza en 0, y pone en los nombres de sus tres
primeras sondas, junto a tres libros, uno por nombre, un enlace simbólico a un fichero de fuera que no
está, uno a uno que está y uno duro a ése, y recifra cada libro una vez; no se sigue ninguno: el que
no estaba no se crea, el que está no cambia —ni sus bytes, ni su dueño, ni su grupo, ni su modo, ni su
fecha de modificación— y los tres nombres siguen como estaban. En las dos, el recifrado de su dueño
sigue, pasa por el paso 1 y se recifra; y, como root, el de un libro asentado de 12345:12346 —en el
nombre ya puesto, con los enlaces simbólicos del dueño, en su directorio, de 12345:12346 con 0700—
rehúsa en el paso 1, con su texto, sin hermanos y con el libro dueño a dueño como estaba, y deja el
cerrojo del libro, de 0:0 con 0600—.
Medido con mutantes, cada uno aplicado en el clon privado, compilado —con un binario distinto del
bueno—, corrido con los doce tests y restaurado después, con sha256, sobre el código del último
borrador, sobre el §821 —en `crates/`, el mismo diff que este sello—: caen
treinta y nueve de los cuarenta, todos en la primera corrida, y sobrevive uno, que se declara
(abajo). El código de antes (los `persistence.rs` y `snapshot.rs` del
§821, con sólo `O_NOFOLLOW`, `ELOOP` y `abrir_sin_enlace` añadidos), en los doce; sin la comparación
previa, en el primero —el recifrado del v1 llega a la migración y da su texto—, en el décimo, que ve
el `snap.*` y el cerrojo de root, de 0:0 con 0600, y en el undécimo, que recifra sin pasar por la
sonda; la comparación después de tomar el cerrojo, en el décimo, por los cerrojos de root, y en el
undécimo, que lo toma; la sonda que se queda en el disco, en el décimo, por la de root junto a cada
libro, y en el undécimo, que recifra; la comparación
previa que sólo mira el directorio del libro, y la que deja fuera `db`, `conf` o `blobs/`, en el
décimo, por el cerrojo y el `snap.*` de root en el `mixto_*` que no mira; crear la sonda sin mirar
antes que es de sled, en el undécimo, que la ve junto a lo que no es un libro; no rehusar si no se
puede borrar, en el undécimo, que recifra; rehusar entonces con `Io`, que `recifrar_e4` reintenta,
en el undécimo, que pasa diez veces por la sonda, con otra cada vez; rehusar si la sonda no se crea,
en el duodécimo, cuyo dueño no recifra, en su (a); la guarda del paso 1 sólo en la migración, en el
duodécimo, en el que root, con el libro de otro, rehúsa más tarde, en el paso 4 —el `.recifrando` no
es del dueño del libro—, con otro texto, en su (b); crear la sonda con `create` en vez de
`create_new`, en el duodécimo, en su (c), donde sigue el enlace a lo que no está, lo crea y, al borrar
la sonda, quita el enlace; sin la guarda del paso 1, en el primero, en el sexto y, como la de
sólo en la migración, en el duodécimo; y con ella después del paso 1 —justo antes de dar el grupo, o al
acabar el paso 2—, en el primero y en el sexto, que dan en la migración, como sin ella, el error inyectado («fallo
inyectado al acabar el paso 1») y no el que nombra a los dos usuarios; en el paso 1, comparar sólo el
directorio, o dejar fuera de la comparación `db`, `conf` o `blobs/`, en el sexto, por su migración;
sin el grupo, en el segundo, el tercero y el séptimo; sin el directorio, en el segundo y en el
séptimo; seguir los enlaces, y quitar la guarda del enlace duro, la del dueño de cada entrada o la de
la copia en 0700, en el cuarto; sin la reserva `& 0o700`, en el tercero; poner los modos antes que el
grupo, en el segundo y en el séptimo, porque el `chown` quita el setuid de `conf` (4640 → 0640): lo
ve sólo ese aserto, el de `comprobar_la_copia`, que comparten los dos; tomar lo de `db` en vez de lo
del homónimo, también en el segundo y en el séptimo, por `conf` (0640 en vez de 4640); no exigir en
el paso 4 el inodo del paso 1, o R vacío en el 1, en el noveno; no bajar a `blobs/`, o no rehusar un
directorio dentro de él, en el octavo; dar el grupo sólo a las entradas con homónimo, en el segundo,
el séptimo y el octavo; que la apertura no mire de quién es el `.recifrando`, en el quinto y en el
sexto; que mire sólo los directorios, o no mire el `.recifrando` por dentro, en el sexto; y que
compare también el grupo, en el séptimo. Con el pid 1 —con `unshare --pid --fork`, con el que el (a) y
el (b) del duodécimo se saltan—, caen también, en el duodécimo y en la primera corrida, los de rehusar
si la sonda no se crea y crear la sonda con `create`, en su (c), y los de la guarda del paso 1 sólo en
la migración y sin ella, en su (d). Sobrevive, 3 de 3 veces, el que hace que la comparación
acepte una lista de dueños vacía, como un borrador anterior: no se alcanza —`de_quien_es` da siempre
el directorio del libro, o un error—, y que ahora rehúse queda deducido, sin test. Los otros treinta
y nueve se corrieron también sobre el borrador anterior, con sus doce tests, y cayeron treinta y ocho
y sobrevivió el mismo; treinta, sobre el de antes, con los diez tests de entonces, y cayeron todos; y
veinticinco, sobre el de antes de ése, sin la comparación previa, con los nueve de entonces, y también
(el asiento). De los nueve que añadió el borrador anterior, cuatro —la comparación previa sin `conf` o
sin `blobs/`, rehusar si la sonda no se crea y la guarda del paso 1 sólo en la migración— pasaban los
diez tests del de antes (lo midió su revisión); y el de `create`, que añade el último, pasaba los
doce del anterior, 3 de 3 veces (lo midió su verificación).
En el clon privado, la capa entera: 511 pasan, 0 fallan y 7 ignorados, sin avisos al compilar, con el
binario del último borrador (146,2 s de tests); y los doce, 10 de 10 veces con ese binario
(9,1-11,0 s), y 3 de 3 con el pid 1 (8,8-9,3 s); y el duodécimo solo, 20 de 20 (5,6-7,1 s). Con
el borrador anterior, 511 pasan y los doce, 3 de 3; con el de antes, 509 y los diez, 3 de 3. Antes de la comparación previa, la capa daba
508 pasan, 0 fallan y 7 ignorados, cinco veces, y los nueve de entonces pasaron 29 de 29 veces con los
binarios de los borradores de entonces (el asiento). Sobre este árbol, el §827 con este sello, y como
root: la capa entera, 515 pasan, 0 fallan y 7 ignorados, sin avisos al compilar (260,2 s de tests),
y los doce, 3 de 3 veces (13,3-17,5 s), y 1 de 1 con `--nocapture`, sin ninguna línea «sin medir».

Medido además fuera de la suite, con `setpriv` y el binario de un borrador anterior, de antes de la
comparación previa, sobre libros como los de la medida de partida: root rehúsa en 12 de 12 —siete
recifrados de un v2 y cinco aperturas con clave de un v1—, con el texto de arriba. Con aquel binario
los siete recifrados rehusaban en el paso 1, después de tomar el cerrojo y de abrir el libro; desde ella rehúsan antes, sin crear el
cerrojo (deducido del código; con el libro sin asentar y sin cerrojo lo mide el décimo test). Después,
el dueño lee las 50 entradas de los diez libros asentados; abre los dos v2 que tenían su cerrojo, con
su saldo —el que no lo tenía no, porque root lo creó suyo, con aquel binario—; los dos v1 de un
directorio de root no los puede migrar, porque no puede crear su `.recifrando` en él; y los cinco de
un directorio suyo los recifra o los migra él mismo, `Ok`, y los abre después, con su saldo. Con
grupo principal 12345 y 12346 de suplementario, el libro sigue en 12345:12346: 12347:12346 lee 15 de
15 entradas y 12348:12345, 0 de 15; antes del arreglo, sobre los mismos tres libros, 5 de 15 y 10 de
15 —los 5 eran los del libro del padre con setgid, que ya conservaba su grupo—. Sin el grupo 12346,
el libro queda en 12345:12345 con 0700 y 0600 —la reserva—, y 12347:12346 lee 0 de 10. En esta
medida todos los libros son de un solo dueño; el que no lo es lo mide el sexto test.

**Lo que no hace:**

- **en la migración, la apertura de sled que precede a la negativa, y el cerrojo que crea antes quien
  abre: es lo que se sigue de que la migración no pueda comparar antes de abrir** —la opción que
  eligió el autor no nombra ningún paso; poner la guarda en el paso 1, y además en `recifrar` antes de
  tomar el cerrojo, es de la sesión, y reversible—. Antes del paso 1 la apertura crea el cerrojo del
  libro si no está (`Cerrojo::tomar`) y abre el libro con sled, porque abrirlo es lo que dispara la
  migración; y sólo migra un libro v1, y que es v1 lo dice lo que guarda sled —datos y ninguna
  cabecera del KDF, `meta:kdf`—, que se lee al abrirlo: sin saber antes que es v1 no hay forma de
  rehusar antes sólo la apertura que lo migraría, y rehusar antes toda apertura con clave de otro
  usuario rehusaría también la de un v2, que no migra, y la opción elegida no dice eso (deducido). En
  el recifrado pasa lo mismo sólo si la comparación de antes no pudo saber quién recifra —la sonda no
  se pudo crear— (con test: el duodécimo, que mide el cerrojo de root que deja). Un borrador del
  duodécimo recifraba como root, por ese camino, tres veces seguidas el mismo libro asentado de
  12345:12346, y en una de sus corridas del banco de mutantes —con uno que no toca ese camino— el
  libro tenía, tras las tres negativas, un `snap.*` de 0:0 con 0644 en el sitio del del dueño (medido
  una vez; se miró tras las tres, y no se sabe tras cuál; con qué frecuencia, no se midió; el
  duodécimo recifra ahora cada libro una sola vez). Por qué, deducido y sin medir: una negativa puede
  escribir en `db` (abajo, 3 de 140) y dejar el libro sin asentar, y una apertura de un libro sin
  asentar deja un `snap.*` (abajo, 40 de 40). Esa apertura de root hace lo que sled hace en cualquier
  apertura; lo que sigue se midió con el código de antes de la comparación previa, cuando también el
  recifrado abría el libro antes de rehusar. Con el libro asentado —abierto y cerrado tras su última
  escritura— no cambia el nombre, el inodo, el dueño, el grupo ni el modo de nada, ni los bytes de
  `conf` y de los `snap.*`, en 420 de 420 negativas, y escribió en `db` en 3 de 140 negativas de
  recifrar un v2 —unos 1,7 KB de bytes distintos— y en 0 de 280 de abrir o recifrar un v1 (con cuatro
  hilos a la vez; lo mismo en dos medidas, con el binario de un borrador anterior y con el de antes de
  un retoque de comentarios y textos, el mismo código que el del borrador anterior a la comparación
  previa; lo que `db` guarda, sus claves y sus valores, no cambia: lo ata el test). Con un libro que
  no está asentado deja un `snap.*` nuevo, de 0:0 y con la umask —0644 con la 0022—, en 40 de 40
  negativas, también en las dos medidas: en las 20 de recifrar un v2, junto a los que había, y
  reescribiendo `db` (entre 506257 y 509254 bytes distintos, en la segunda); en las 20 de abrir con
  clave un v1, en el sitio del `snap.*` del dueño, que ya no está; y las 20 de recifrar después ese
  v1, ya asentado por esa negativa, no cambian nada. Que el dueño lea ese 0644 se deduce del modo; no
  se midió. Con la umask 0077 —o cualquier umask que quite la lectura a otros, como la 0027 para quien
  no es del grupo 0 (deducido)— el `snap.*` nuevo es de 0:0 y el dueño no lo lee —con la 0077, 0600—,
  y ya no abre el libro («IO error: Permission denied (os error 13)», en 2 de 2 en las dos medidas: un
  recifrado, que reescribió `db` —de 17561 a 524287 B en la segunda—, y una migración, que rehusaron;
  desde la comparación previa, el recifrado rehúsa antes y no deja nada: lo mide el décimo test) hasta
  que root se lo dé —lo midió la revisión de un arreglo anterior, descartado, con un abridor mínimo de
  sled: «Ok» tras el `chown` de ese `snap.*`— o lo quite (deducido, sin medir). Lo mismo haría
  cualquier apertura de root sobre ese libro, también la sin clave del cli (deducido: es la misma
  apertura de sled; sin medir). Y si no había cerrojo, el que crea root deja al dueño sin abrir
  (abajo). **La alternativa, comparar en `recifrar` antes de tomar el cerrojo y de abrir el libro,
  está hecha** (arriba): lo evita en el recifrado, y no en la migración. Su coste fue una segunda
  comparación —la del paso 1 sigue haciendo falta para la migración, y de respaldo—, la sonda, y
  cambiar el primer test, que contaba tres veces `recifrar:0` y ahora cuenta una. No pidió `libc`: el
  usuario de quien recifra lo da el dueño de la sonda, como en el paso 1 lo da el dueño de R. Un
  borrador anterior de este texto decía que lo pedía, y que eso era superficie nueva; no lo era (lo
  vio su verificación). Hacerla es de la sesión, y reversible; el autor eligió su opción antes de que
  se midiera esto;
- **la sonda**: se crea junto al libro —en su directorio padre, donde el paso 1 crearía R— porque su
  dueño es el usuario con que el proceso crea en ese directorio, el que tendría R, también con un NFS
  con `root_squash` o con un montaje que cambie los dueños, y porque un fallo de permisos al crearla es
  el mismo que el del paso 1 (deducido). No `/proc/self`: da el usuario efectivo, y no el de los
  ficheros (`setfsuid`); sólo está en Linux con `/proc` montado; y no sabe del sistema de ficheros del
  libro (deducido; sí da el usuario efectivo tras un `setuid` en el mismo proceso, que lo deja sin
  volcado: medido en la sesión, en Linux 6.18, con su `status` ya de root). Ni el directorio temporal,
  que puede ser otro sistema de ficheros, y una sonda que se quedara allí no se vería junto al libro
  (deducido). Crearla y borrarla cambia la fecha de modificación del padre (medido en la sesión, en
  Linux 6.18 y ext4, con un arnés fuera del árbol: el recifrado de root que rehúsa el libro de otro,
  v2 o v1, la cambia sin dejar nada en el padre; y el que rehúsa un directorio que no es de sled, no),
  como la cambia R (deducido). Junto a lo que no es de sled no se crea: se mira antes, sin escribir, y
  rehúsa como antes del §828 (con test, el undécimo; un borrador anterior la creaba antes de mirarlo, y
  lo vio su revisión). Si no se puede borrar —un padre con `chattr +a`, donde se crea y no se borra—,
  rehúsa con `Malformed`, nombrándola, sin tomar el cerrojo ni abrir el libro, y se queda hasta que se
  borre a mano (con test, el undécimo; un borrador anterior la dejaba sin decirlo, y lo midió su
  revisión). `Malformed`, y no `Io`, porque quien reintenta un `Io` crearía otra sonda en cada intento
  (medido: con un estado intermedio de este arreglo, que rehusaba con `Io`, el `recifrar_e4` de los
  tests, que lo reintenta hasta diez veces, dejó diez con el arnés; y con el mutante que rehúsa con
  `Io`, el undécimo test pasa diez veces por la sonda). Si el proceso muere entre crearla y borrarla, también se queda —vacía, de quien
  recifraba, con 0600— y nadie la mira ni la borra: se borra a mano (deducido, sin test). Si la sonda
  no se puede crear —quien recifra no escribe en el padre, el nombre ya está, o no cabe—, la
  comparación de antes no rehúsa, y queda la del paso 1, con lo que deja esa apertura (con test, con
  un nombre en el que no cabe y con el nombre ya puesto, por enlaces: el duodécimo). Con el nombre ya
  puesto —diez sondas del dueño del libro, o diez enlaces, con el pid del proceso—, root rehúsa así en
  el paso 1, deja en el libro, sin asentar, un `snap.*` suyo —con la umask 0077, 0600— y, junto al
  libro, su cerrojo, y el dueño ya no lo abre (medido con el arnés). Un enlace no se sigue: ni uno
  simbólico, a un fichero que no está o a uno que está, ni uno duro; el que no estaba no se crea, y el
  que está no cambia (con test: el duodécimo, con el libro del proceso y, como root, con el de otro
  dueño y los enlaces de éste; antes, los dos simbólicos, medidos con el arnés). Si quien recifra no puede
  escribir en el padre, el paso 1 tampoco podrá crear R (deducido);
- **los `snap.*` que escribe sled al reabrir el libro en el paso 8**, y en cualquier apertura del dueño
  en su día a día: son de su grupo principal —o del grupo del padre, si tiene setgid— y con su umask,
  no los del libro (medido: un `snap.*` de 12345:12346 con 0644 en un libro 2750, y uno de 12345:12345
  con 0644 en uno 0700, tras recifrar o migrar su dueño). Con un directorio 0750 o 0700 no alcanzan a
  nadie más que a los que ya lo atravesaban (deducido);
- **el cerrojo del §816** que no estaba: quien abre lo crea suyo, y quien recifra también, si es el
  dueño del libro o si la comparación de antes no pudo saber quién es (lo mide el duodécimo test); uno de root deja al dueño sin
  abrir (medido de nuevo, con el código de antes de la comparación previa: «….cerrojo: Permission
  denied»); ya lo dice la 3.14, en su residuo;
- **lo que ya recifró o migró el código de antes** sigue de quien lo hizo. Un `chown -R` a su dueño le
  devuelve el dueño, y quita el setuid de un fichero que lo tenga, que hay que volver a poner (medido
  en la sesión: tras el `chown -R` de root, un `conf` 4640 queda en 0640, y el directorio 2750 se
  queda; que con eso el dueño lo abra, deducido). Un `.recifrando` de otro que dejó a medias hace
  rehusar la apertura, con el texto que dice qué hacer; y un `.retirado` junto a un libro de otro
  dueño —una muerte de root tras su paso 6— la apertura lo borra como antes, y el libro queda de quien
  recifró (deducido, sin test);
- **lo que el dueño no controla del grupo**: si no es del grupo del libro, el libro recifrado o
  migrado queda en el grupo con que nació la copia —su grupo principal, o el del padre con setgid—,
  sin bits de grupo ni de otros, y el grupo del libro ya no lo lee; recuperarlo pide un `chgrp` de
  quien pueda. No toma las ACL POSIX ni los demás atributos extendidos del libro —R toma la ACL por
  defecto del padre— (deducido, sin medir);
- **lo que no alcanza la guarda de los enlaces**: `std` no tiene `openat` ni `fchownat`, y cada mirada
  resuelve la ruta otra vez; un cambio que se deshace entre dos miradas no se ve, y dentro de R sólo
  puede hacerlo su dueño, o root. Quien puede escribir en el directorio padre puede cambiar R por
  otro directorio —para eso, el padre no tiene el bit sticky, y el directorio que pone ya está en ese
  padre, porque llevar un directorio a otro padre pide escribir en él (deducido de `rename(2)`)—.
  Hasta el paso 4 la guarda lo ve, con test: el R que no está vacío al compararlo, en el paso 1, y el
  que tiene otro dispositivo u otro inodo, en el 4; y el recifrado que rehúsa borra lo que haya en el
  sitio de R, como desde el §812. Uno vacío del dueño del libro, puesto antes de compararlo, recibe la
  copia y el grupo como el de verdad (deducido). Y queda el hueco entre la comprobación del paso 4 de
  que R es el que nació y la vuelta por sus entradas, que lee R por su ruta (`read_dir`) y abre cada
  entrada por su ruta: si en ese hueco alguien pone otro directorio en el sitio de R, las entradas de
  ese otro que sean del dueño de R —el del libro, que es quien recifra— y, si son ficheros, con un solo
  nombre, toman el grupo y los modos de su homónimo en el libro, o los de `db`, y lo que lleve un
  directorio suyo de dentro, igual; las demás hacen rehusar. Al acabar las entradas la guarda ve el
  cambio y rehúsa, pero después de darlos; al directorio puesto no le da el grupo ni el modo del
  libro, y el recifrado que rehúsa borra lo que haya en el sitio de R, si quien lo puso no lo quitó
  antes. Desde este sello le da también el grupo del libro; hasta él, `copiar_modos` daba por la ruta
  sólo los modos, a lo que hubiera allí, de quien fuera, desde el paso 4 hasta el 6. Lo puede hacer
  quien escribe en el padre, con un directorio que ya esté en ese padre, como arriba; y lo que gana no
  es nada que no fuera ya del dueño del libro —lo de quien hace el cambio, si no es el dueño, hace
  rehusar—: que lo del dueño que hubiera en ese directorio tome el grupo y los modos del libro, y que
  lo lea el grupo del libro si puede entrar en él, porque a ese directorio no se le cambia el modo
  (todo, deducido del código, sin test). Después, desde que acaba el paso 4 hasta el 6, cambiarlo es el
  hueco que ya tenía el §812 (deducido, sin test);
- **lo que se rehúsa de más**: el libro que no es de un solo dueño no lo recifra ni lo migra ninguno
  de los dos —con test el del directorio; el de los ficheros, deducido—, aunque al dueño de los
  ficheros le bastaría (deducido): que sea todo de un dueño lo arregla. Y, deducido y sin medir: en un
  sistema de ficheros que no guarda los modos de Unix, R no queda en 0700 y el recifrado rehúsa; con
  NFS y `root_squash`, R nace de otro usuario y root rehúsa —en el recifrado, ya en la comparación
  previa, porque la sonda nace igual—;
- fuera de Unix no cambia nada, y no está probado en Windows ni en NFS.

Se redactó en privado, como pide la sección 5. Cómo se publica lo eligió el autor (21-10-2026, en el
chat), a otra pregunta de la sesión: la opción «Como el §812, sin GHSA (Recomendado)», cuya
descripción, escrita por la sesión, decía: «De una vez con el código. El aviso es la sección nueva de
SECURITY.md (la 3.17) y el asiento, sin GHSA ni release, como el §812, el §816 y el §817.» Que la
sección 5 no pida aviso ni release, porque el kit publicado no depende de la capa, es lectura de la
sesión. `AUDITORIA.md` §828.

### 3.18 Ocho defectos del verificador del kit, de la 0.1.0 a la 0.4.3, y uno de la capa: VERDE a sobres que no se sostienen, 101 ante una prueba mal formada, un coste que elegía quien envía y una emisión delegada que se reenviaba — ⚠️ MEDIDO, cerrado en el kit 0.4.4 y en la capa (§711 a §723)

El verificador del kit, `zk-ssl-verify`, tenía ocho defectos en las releases publicadas. Cada uno
alcanza a las suyas, dichas en su punto. La capa tenía además uno suyo, el de la emisión delegada,
que va detrás de los ocho y sin número: no es del kit ni del aviso. Salieron de una revisión
interna del autor con la ayuda de Claude Code (`GENAI.md`). Se midieron antes de publicar nada, con
el kit compilado de cada árbol y, donde se dice, con el binario publicado. Cada asiento dice qué
midió y con qué. Los puntos llevan el número del aviso de seguridad de la 0.4.4.

1. **Una prueba mal formada hacía salir al kit con 101** (§711). De la 0.3.0 a la 0.4.3. Upstream
   valida la cabecera de una prueba con `assert!`, y los bytes los pone quien la envía. Medido
   antes del arreglo: de las mutaciones de la cabecera, 47 de 464 entraban en pánico; de las de la
   forma, 85 de 521; y una capa FRI de menos, también. En el nodo, un pánico con el candado tomado
   lo llevaba a PARADA. Es otra vía que la de la 3.7, que abortaba por memoria. **Cerrado**: el fork
   valida lo que lee antes de construir, y el kit da ROJO con su causa, `prueba mal formada: …`.
2. **El mismo `mmrSize` con distinto `epochDigest` daba VERDE** (§712, §714). En la 0.1.0, el sobre
   de extensión; desde la 0.2.0, también el de consumo; desde la 0.3.0, también el ancla en modo 3.
   Con el mismo tamaño, la consistencia del MMR es la identidad: dos cabezas firmadas en el mismo
   sitio de la historia, con contenidos distintos, pasaban. Y con la misma clave y un `mmrSize`
   mayor, nada miraba el índice embebido. **Cerrado**: `extension: mismo mmrSize y digest
   distinto` y `extension: el indice embebido no crece`. El ancla gana el modo 5, que delata esas
   dos cabezas: `VERDE: BIFURCACION` (§714). El testigo agrupa por el índice que la firma acredita y
   se detiene con la clase `bifurcacion`.
3. **El resto FRI no se comparaba con su compromiso** (§713). De la 0.3.0 a la 0.4.3. `winter-fri`
   0.13.1 mete el compromiso del resto en la moneda, y su verificador no lo comprueba: quien prueba
   podía elegir el resto después de ver las consultas. El alcance en solidez no se cuantificó.
   Medido: con las opciones de la casa, el plegado contrasta el resto con 35 a 40 posiciones.
   **Cerrado** en el canal del fork y en la segunda implementación:
   `FriVerificationFailed(RemainderCommitmentMismatch)`.
4. **Una prueba admitía más de una escritura** (§711, §715). De la 0.3.0 a la 0.4.3. Otros bytes de
   la misma prueba verificaban igual: una longitud con bytes de más, una fila o un grupo de más,
   una capa FRI de más, o el número de restricciones con un múltiplo de `2^32` sumado. El veredicto
   sobre el enunciado no cambiaba, pero la huella de los bytes no identificaba la prueba, y la
   guarda del §654 en la capa se esquivaba (el matiz de la 3.4). **Cerrado**: una sola escritura,
   en el lector del fork y en el canal del verificador; y la capa guarda además la huella del
   contenido.
5. **El juez de la edad trabajaba en proporción a `2^m` antes de mirar la prueba** (§717, §721). De
   la 0.3.0 a la 0.4.3. `m` sale del `nextPending` de una cabeza que el mando verifica con la clave
   que ella trae, así que la elegía quien enviaba el paquete. Medido antes del arreglo: 63,2 s y
   531 MB con `m = 23`. **Cerrado**: `M_MAX` vale 13, el `m` más alto que cabe en las dos cotas del
   RFC-0007 con el probador de producción. Una `m` mayor es ROJO antes de leer la prueba,
   `edad: m = N fuera de 3..=13`, en milisegundos. El probador aplica el mismo tope.
6. **Un acta de rotación con un `mmrSize` no canónico** (§718). Sólo la 0.4.3, la primera que lee
   actas. Severidad baja. Su lector no aplicaba la cota `< p`, y la firma sólo ata el elemento
   reducido: un acta firmada con `mmrSize = N` daba VERDE presentada con `N + p`, medido con el
   binario de la 0.4.3. No rompe la conservación ni suplanta a un operador. **Cerrado**:
   `actas[i]: mmrSize: … no es canonico`. La misma pasada lleva la regla de la 3.8 a las fronteras
   que faltaban: el juez, el almacén, la instantánea y el testigo.
7. **La banda y el cobro pendiente acotaban el saldo a 63 bits, no a 62** (§723). De la 0.3.0 a la
   0.4.3, leído en sus commits. Severidad baja. El RFC-0017 bajó a 62 bits las AIR vivas (§641,
   §719); las dos del kit se quedaron en 63. Con una hoja de saldo en `[2^62, 2^63)`, la prueba de la
   banda verificaba un rango que la hoja no cumple. Medido con el kit de antes del arreglo: un sobre
   de rechazo `InsufficientBalance` con su cabeza firmada daba VERDE. La capa no firma esa hoja: su
   carga y su instantánea la rechazan, medido. Sólo la alcanza quien firma una raíz fuera de la
   capa. **Cerrado**: la v2 de las dos AIR fuerza a cero el bit 62 de cada segmento. Una prueba v1
   vale con el superior desde `2^32 - 2`, o si es la ya publicada, que el kit reconoce por su
   huella; por debajo, salida 3. La v2 cambia las pruebas que viajan sin subir `zkssl/0.4`: como la
   ruptura del RFC-0017 (§826), la paga el corte `zkssl/0.5`. Lleva la fe de erratas de la D-3 del
   RFC-0017, redactada con el arreglo.
8. **Dos operadores de acuerdo refutaban un conflicto genuino sin que nada lo delatara** (§722).
   Sólo la 0.4.3. Severidad media. La sección 2.5 de `spec/PAQUETE.md` decía que no podían firmarse
   una historia común, y era falso. Medido antes del arreglo, con dos nodos reales: un libro nuevo
   copia el diario de otro, se declara su heredero con un acta, y su cabeza refuta el conflicto.
   **Cerrado** en lo que se puede ver: la `procedencia` de las actas fija el relevo en el MMR. Los
   ocho sitios que exigen continuidad —la extensión, el consumo y las seis cabezas de la
   completitud— exigen que cada cabeza caiga en el relevo de su tramo, por los dos bordes:
   `RELEVO: …`. El sobre del solapamiento da VERDE a una cabeza firmada fuera del relevo de su
   tramo. El nodo gana `--relevo-minimo`.

**Y el de la capa: una emisión delegada se reenviaba cuando el estado volvía atrás** (§716). La
capa verificaba la emisión delegada y la emisión a pendiente contra un compromiso que sólo ata el
estado —las raíces de antes y de después, el importe, el suministro y el tope—, sin contador ni
registro de lo ya aplicado. Una quema, o una desemisión, devuelve la raíz y el suministro a los de
antes de emitir, y entonces la misma autorización de los custodios volvía a emitir. Medido antes del
arreglo: emitir 250.000, quemar y reenviar daba `Ok`, con dos entradas `Mint` y una sola
autorización; en bucle, tres reenvíos aceptados, 750.000 redimidos y 250.000 en saldo con una
autorización. Donde se anotó, el agregado cuadraba: es un fallo de autorización, no de
conservación. La capa no tiene release, y el nodo no expone la emisión delegada, salvo `dev_fund`
con `--dev`. **Cerrado**: lo que firman los custodios lleva el contador de emisiones de la capa, y
un reenvío es `AutorizacionConsumida`. Una autorización firmada con el formato de antes, sin
contador, ya no emite: `StaleState`.

**Alcance.** Ninguno de los ocho del kit crea saldo ni gasta sin autorización. El 2 y el 8 rompían
la detección: el kit daba VERDE a una historia que no se sostiene, o no delataba la fabricada. El 3
quitaba una comprobación del verificador STARK, sin alcance medido. El 7 daba un VERDE falso sobre
lo que dice la hoja comprometida. El 1 y el 5 eran de disponibilidad: un 101 en vez de ROJO, y un
coste que elegía quien envía. El 4 y el 6 dejaban que los bytes, o el entero, dijeran otra cosa que
lo firmado. El de la capa sí emitía sin autorización nueva: una de los custodios valía para más de
una emisión.

**Residuo**:

- **La refutación fabricada sigue VERDE** (8). Prueba una historia de cabezas, no de estado. La
  detección pide una cabeza de la clave que se fue firmada desde su relevo, y que alguien la
  guarde. Alternar los dos libros, rellenar el MMR con el contador saltado o volver a firmar el
  relevo no dejan evidencia que un sobre juzgue. El testigo no aplica el relevo. Y el relevo delata
  a un operador honrado que rota desde un diario restaurado sin seguir la D-G del RFC-0015.
- **El alcance en solidez del resto FRI** (3) no se cuantificó.
- **La CPU del juez de la edad sigue lineal en `2^m`** hasta el tope (5). Un libro que pase de
  `2^13` posiciones de pendiente no tiene prueba de edad.
- **La v1 de la banda se acepta desde la cota** (7). La cota está derivada y su frontera probada en
  enteros, no buscada contra el circuito con un probador. Ningún vector ejercita el cobro pendiente
  por el mando con un superior pequeño.
- **La huella del contenido no persiste** (4). Tras reiniciar el nodo, la defensa es el lector y el
  canal del fork. El cierre de fondo es el nonce del tren `zkssl/0.5`.
- **En `wasm32-wasip1` la red del pánico no recoge nada** (1): allí un pánico aborta.
- **Volver atrás el almacén entero de la capa**, registro incluido, devuelve el contador, y el
  reenvío vuelve a valer (§716). Es la clase de la bifurcación (2): la capa no puede saber que su
  disco es viejo. Congelar, gobernar y recuperar, que ya llevaban su contador, no ganan la causa con
  nombre: su reenvío cae con `StaleState`.
- **Upstream.** El 1, el 3 y el 4 están también en winterfell 0.13.1, del que viene el fork. Su
  mantenedor recibió el aviso el 22-10-2026, con los tres parches y sin plazo acordado; en
  upstream todavía no están arreglados.

**Lo que conviene hacer.**

- Actualizar a la 0.4.4.
- Un VERDE de la 0.4.3 o anterior sobre una extensión, un consumo o un ancla con el mismo
  `mmrSize`, sobre un acta, sobre un rechazo `InsufficientBalance` o sobre una extensión, un consumo
  o una completitud entre dos claves se vuelve a juzgar con la 0.4.4.
- Una salida 101 de una release anterior es un rechazo, nunca un VERDE.
- Quien rote la clave de un nodo reconstruye antes el diario (la D-G del RFC-0015) y arranca con
  `--relevo-minimo`.
- Quien use la emisión delegada de la capa pide de nuevo a los custodios las autorizaciones
  firmadas con el formato de antes: ya no emiten.

Se preparó sin publicar y se publica de una vez con su arreglo, como pide la sección 5, con la
enmienda del §724. Con este sello salen, como piden la sección 5 y `spec/rfc/PROCESO.md` §«Fallos
de seguridad», la release `arqueo-verify-v0.4.4` y su aviso de seguridad, que nombra las afectadas
—de la 0.1.0 a la 0.4.3, cada punto con las suyas— y la primera que lo arregla, la 0.4.4. Los
publica el autor: hasta el asiento que registre la release (§726), la vigente sigue siendo la 0.4.3
(`doc/KIT.md` §0). El mismo acto corrige también la capa, sus circuitos, el nodo y el testigo, que
no tienen release: su aviso es esta sección y sus asientos, del §711 al §723. `AUDITORIA.md` §711 a
§725.

## 3.bis La superficie de protocolo (§197-§201): qué añade y qué defiende

Desde agosto de 2026 esto no es solo una capa: hay cable, nodo, SDK y un
contrato público. Una superficie nueva es, por definición, riesgo nuevo —
y también trae dos defensas que antes no existían.

**Riesgo que añade** (arriba, §3.3): un puerto abierto sin auth ni TLS.

**Lo que el diseño protege, verificado en código:**

- **Las claves no salen por la API y, desde el §538, tampoco salen literales
  en la prueba** (§521, §523; §538). El pago va en dos fases y la prueba se
  genera **en local** (`prove_send`/`prove_claim`); `Wallet::spend_key` es
  privado y **ni siquiera implementa `Serialize`**: no hay accidente posible
  por serialización. Eso sigue siendo cierto. Medido en las sesiones 162 y
  163 (20 y 21 de septiembre de 2026): las pruebas STARK de la casa con
  winterfell 0.13 **no ocultaban su testigo**. Cada prueba abría 42 filas de
  su traza en claro, y lo que iba en esas filas se publicaba: la clave de
  gasto en el envío, el cobro y la prenda —y en el envío v2, además, el
  saldo, el importe, la sal, el `leaf_salt` y la `X`—; el saldo y el
  `leaf_salt` en la banda; el emisor en la edad, cuando todos los pendientes
  son del mismo; y la sal en los dos sobres portables, con la `X` y el
  importe exacto en el de cobro y el `delta` y el `refund_id` en el de pago.
  La regla que lo explicaba se midió con dos falsadores (§523): **salía
  literal todo valor que va en una columna constante de la traza**. Por eso
  la auditoría —un circuito para los tres modos— publicaba la clave de gasto
  y el saldo exacto; la quema, la clave; el envío, la identidad del
  receptor; y **cada autorización delegada, la clave de su custodio**: tras
  una sola emisión, congelación o recuperación delegada, el nodo tenía las
  claves de dos custodios y podía autorizar la siguiente. La gobernanza
  usaba el mismo circuito: deducido, no medido. La conservación no caía;
  caía quién podía mover el suministro. La solidez tampoco caía: cada prueba
  seguía probando lo que decía. Lo que caía era la ocultación, y con ella la
  custodia de las claves —la de gasto y las de custodio— frente a quien
  viera una prueba. **Desde el §538 (RFC-0009 E3b-2, `zkssl/0.4`) el
  probador oculta el testigo**: los 23 probadores con fila en la tabla del
  RFC-0009 producen con filas aleatorias y cociente cegado sobre un fork de
  winterfell 0.13.1 (`crates/winter-*`), y lo que se promete es lo que la
  suite de E2 mide en cada canon: **una prueba que verifica demuestra la
  transición que enuncia y no publica literal ningún valor de columna
  constante de su traza** (cero literales en 100 celdas y 22 filas). Lo que
  no se promete: que nada se deduzca de lo que se abre; la cota vive en el
  RFC-0009 (D-I: 44 aberturas por columna frente a T ≥ 64 filas aleatorias)
  y ni el fork ni la construcción están auditados (H7). Quien ve una prueba
  sigue siendo el nodo, que las recibe, y quien lea un vector o un paquete
  publicado: los de `spec/vectors/` de la 0.4 no llevan ningún literal del
  testigo; los de la 0.3, conservados bajo `spec/vectors/0.3/`, siguen
  llevando lo que llevaban, y del catálogo de rechazos de la 0.3 siguen
  derivándose dos claves de gasto de sandbox. La regla del API —la clave no
  viaja— se cumple desde el §538 en lo que la suite mide; entre el §521 y el
  §538 no se cumplía, y los asientos lo dicen.
- **El cable rechaza lo que no entiende**: DTOs con
  `deny_unknown_fields`, hex canónico, digests de anchura fija.
- **Los vectores de conformidad son una defensa, no solo documentación.**
  Una segunda implementación que divergiera —en la hoja, en el orden, en
  el encadenamiento— **no pasaría `conformance --check`**: la divergencia
  se vuelve detectable campo a campo en vez de silenciosa.
- **El wallet en reposo** (§199) usa la misma construcción que el ledger
  con **dominio propio**, y un test exige que la clave del ledger **no**
  abra el keystore. Desde el §702 (RFC-0001) la clave de los dos se deriva
  con **Argon2id** (RFC 9106: 64 MiB, tres pasadas, cuatro carriles) y una
  sal de 16 bytes por fichero, guardadas en claro junto a lo cifrado; un
  keystore `zkssl-keystore/1` o un libro cifrado de antes, abiertos con su
  frase, se migran a la versión 2. ⚠️ **Corregido en el §702**: hasta
  entonces este párrafo decía «Su KDF es SHA-256, que **no** es una función
  de derivación de contraseñas: una contraseña débil es forzable», y lo era:
  sin sal ni coste, cada intento fuera de línea costaba un SHA-256. Lo que
  sigue en pie: una contraseña débil sigue siendo débil —el coste encarece
  cada intento, no reduce los intentos—, y quien tenga el proceso en marcha
  tiene la clave derivada en memoria. ⚠️ **Corregido en el §812**: hasta aquí
  este párrafo no decía lo que quedaba en el disco, y un libro migrado
  guardaba en sus ficheros lo sellado con la clave v1, que se forzaba a la
  velocidad de un hash (§3.14). Desde el §812 la migración no lo deja, y
  `SovereignLayer::recifrar` cambia la frase y la sal del libro entero.

---

## 3.ter Frente a los sistemas que ya existen — comparación honesta

> Esta tabla existe porque un lector serio va a hacerla mentalmente de
> todas formas. Se hace aquí, con las dos direcciones puestas: **hay tres
> ejes donde este diseño va por delante y cuatro donde los otros lo
> aplastan**. Los datos ajenos llevan fecha y fuente al pie; los propios,
> referencia a `AUDITORIA.md`.

### Donde este diseño va por delante

**1. Autoridad de gasto post-cuántica — hoy, no en un roadmap.**

| sistema | qué autoriza un gasto | estado frente a Shor |
|---|---|---|
| Bitcoin | ECDSA / Schnorr (secp256k1) | vulnerable. BIP-360 (P2MR) se fusionó en el repositorio de BIPs el 11-02-2026 — hito de documentación, **no activación**; BIP-361 (retirada de firmas legadas) es de 14-04-2026 y polémico. ~6-7 M BTC (25-33 % del suministro) tienen la clave pública ya expuesta |
| Ethereum | ECDSA en cuentas, BLS en validadores | vulnerable. La Fundación creó un equipo PQ en enero de 2026; `leanXMSS` (firmas hash) + `leanVM` en desarrollo, con horizonte declarado hacia 2029 |
| Zcash | firmas sobre curvas (Pallas/Vesta) | vulnerable |
| Solana | Ed25519 | vulnerable |
| **Arqueo** | **conocimiento de preimagen**: identidad, salt de hoja y autoridad derivan de la clave **por hash** (§117) | **no hay firma clásica en la vía de pago**, y STARK/FRI solo usa hashes: no hay curva que romper |

⚠️ **La reserva que toca hacerse**: «post-cuántico» aquí significa *sin
supuestos de curva*, no *invulnerable*, y el nivel frente a un adversario
cuántico **no está medido** (§3.11). Lo medido es clásico: 127 bits
conjeturados —112,8-116,8 según el circuito con el término DEEP, que la
fórmula no descuenta, y menos en la edad— y, demostrables, 59 en UDR y
80-88 en LDR según el circuito, menos en la edad con `m` alta (§3.11). El
techo de **63 bits de solidez** que este proyecto midió y publicó
(hallazgo 3) es el de una configuración sin extensión de campo; la de
producción usa la cuadrática. Un sistema con miles de validadores y años
de producción sigue siendo, hoy, **más seguro en la práctica** que uno sin
auditar. ⚠️ **Corregido en el §698**: hasta entonces este párrafo decía
que «su configuración por defecto tiene techo de 63 bits de solidez sin
extensión de campo», y no decía que el nivel cuántico no está medido. ⚠️ **§838**: «sin supuestos de curva» vale para la vía de pago y para lo que se firma, no para el transporte. El TLS del cli y del SDK negocia un ECDHE clásico y verifica firmas ECDSA P-256/P-384 y Ed25519 de certificados y del handshake, y la clave de vista viaja bajo él (sección 3.3): es la excepción del §106 para el transporte, declarada en el §838 de `AUDITORIA.md`.

**2. Sin ceremonia de confianza — y sin haberla tenido nunca.** Zcash la
eliminó con Halo 2 en Orchard (mayo 2022), pero Sprout y Sapling nacieron
de ceremonias y esos pools legados existen. Aquí el arranque **no genera
claves**: 1,47-1,85 ms medidos (⚠️ §841: decía «0,67 ms»; remedido en tres corridas de `metrics_of_the_layer` sobre el §840, en la tabla de `ARQUITECTURA.md` «Métricas de la capa, medidas»), no hay secreto que destruir ni participante en
quien confiar. *Matiz honesto: esto no diferencia frente a Bitcoin o
Solana, que no usan SNARKs en absoluto — diferencia frente a la familia
Groth16/PLONK-KZG, que fue de quien se descartó.*

**3. Supervisión demostrable con revelación ACOTADA.** La clave de vista
de Zcash da **acceso de lectura** a quien la tenga: es todo-o-nada sobre
lo que cubre. Aquí el titular produce una prueba de **banda** —«estoy
entre X e Y»— que el supervisor verifica **sin acceso al ledger** y sin
aprender el saldo; y el tope de emisión está **atado en el circuito**,
no en la política de un cliente. Bitcoin, Ethereum L1 y Solana no
ofrecen privacidad de contenido en absoluto, así que la comparación en
este eje solo aplica contra Zcash.

### Donde los otros aplastan a este proyecto

| eje | ellos | Arqueo |
|---|---|---|
| **Descentralización** | miles de validadores/mineros independientes | **UN nodo, un operador**. Ve el estado, ordena, puede censurar |
| **Rendimiento** | Solana en miles de TPS; Bitcoin y Ethereum en un orden muy superior a este | **1,5-1,9 TPS** medidos (§123) |
| **Madurez** | años en producción, auditorías repetidas, recompensas por fallos | **cero auditorías externas**, prototipo de investigación |
| **Tamaño de prueba** | Groth16: 192 B | **72.382-84.244 B** por prueba de envío o de cobro, ocultas, la banda que ata un test desde el §538 (un pago son dos: 145.953-167.967 B); el tamaño de las demás pruebas no lo ata ninguna banda — el precio de no depender de nadie. Antes de ocultar, el envío y el cobro medían 64,6 y 65,3 KB (§218). Los 39.048 B de las tablas comparativas (⚠️ §806: decía «Los 36,7 KB»; remedido el 2026-10-14 sobre `b6491ef`, en `FIVE_BACKENDS.md` §4) son del circuito de comparación, con otras opciones y sin ocultar |

**Y una lección que este proyecto toma prestada, no presta**: el fallo de
sub-restringimiento de Orchard (junio de 2026) ocurrió en la clase que
aquí figura como **§3.1, prioridad más alta**. No se cita para señalar a
nadie: se cita porque **valida el orden de prioridades** de este
repositorio y porque su remedio —verificación formal— es la escalera que
aquí está a medio subir. Cuando un sistema auditado y maduro tarda cuatro
años en ver una de estas, la conclusión correcta no es «a nosotros no nos
pasará»: es **«nuestros circuitos tampoco están probados, y lo decimos»**.

---

## 4. Alcance

**En alcance**: solidez de circuitos y de la función de transición;
autorización de gasto; corrección del verificador; propiedades de privacidad
declaradas; integridad del registro encadenado.

**Fuera de alcance hoy**, por diseño o inmadurez: consenso distribuido;
resistencia a un operador malicioso sin observadores externos; recuperación
ante pérdida del nodo; análisis de metadatos; seguridad económica de un token
—el proyecto **no requiere ninguno**—.

## 5. Cómo reportar una vulnerabilidad

**Fallos de solidez** —crear saldo, gastar sin autorización, o hacer que el
verificador acepte una transición inválida— repórtalos **en privado** antes de
divulgarlos.

Usa **«Report a vulnerability»** en la pestaña **Security** de este
repositorio. Abre un aviso privado que solo ve el mantenedor; no expone
ninguna dirección de correo.

**Problemas no sensibles** —documentación, límites ya listados, mejoras—:
*issue* normal.

**Qué incluir**: descripción del fallo, la lógica que lo produce y, si es
posible, un caso mínimo que lo demuestre —por ejemplo, una traza inválida que
el verificador acepta—.

**Qué esperar**: proyecto de investigación sin equipo dedicado. No hay
compromisos de respuesta ni recompensas. Se agradece la divulgación
responsable y se da crédito a quien lo desee.

**Qué sale cuando se arregla**: cada fallo arreglado en el kit publicado —las
releases `arqueo-verify-v*`— sale con su **aviso de seguridad**, un *GitHub
Security Advisory* de este repositorio que nombra las versiones del kit
afectadas y la primera que lo arregla, con el crédito de quien lo reportó si lo
desea.

**Cómo se arregla**: en privado —en el fork privado temporal que se crea desde
el borrador del aviso o en un repositorio privado del mantenedor (también un
clon sin remoto), nunca en una rama, un PR o un *issue* públicos— y se publica
de una vez: el código, los vectores, el RFC o la enmienda con su etapa ya
construida, el asiento de `AUDITORIA.md`, la release y el aviso. Lo preparado en
un repositorio privado entra en `main` por avance rápido, un commit por arreglo
con su asiento, justo antes de la release y el aviso, si los lleva. Los vectores
viejos no se reescriben: si el arreglo cambia el cable, la versión sube y los
viejos se conservan bajo la suya (regla 2 de `spec/rfc/PROCESO.md`). Lo escribe
`spec/rfc/PROCESO.md` §«Fallos de seguridad», que dice también por qué el RFC
de un arreglo de seguridad no pasa antes por el árbol, y que el borrador de un
reporte a otro proyecto no entra en él.

**No** se harán afirmaciones categóricas sobre la seguridad del sistema en
respuesta a un reporte: se corregirá, o se documentará el límite.

---

## 6. El consenso es el último intermediario: el caso a favor del dinero cuántico

Conviene terminar una política de seguridad respondiendo a la pregunta que
la ordena entera: **¿de quién hay que fiarse todavía, y por qué?**

**Cada uno de estos sistemas es una máquina de eliminar intermediarios.**
Bitcoin quitó al banco emisor. Zcash quitó al observador. Ethereum quitó
al anfitrión de la aplicación. Este proyecto quitó a los participantes de
una ceremonia de setup —los que, coludiendo, podrían crear dinero sin
dejar rastro—. Todos quitaron a alguien. **Ninguno quitó al que ordena.**

Porque eso es el consenso, dicho sin liturgia: **un intermediario de
orden, hecho plural y hecho caro.** Los mineros y validadores no son la
ausencia de un tercero; son un tercero repartido entre muchos, que sigue
decidiendo qué entra, en qué posición y qué se queda fuera. De ahí salen
la censura, el reordenamiento y la extracción de valor por orden (MEV):
no son patologías del consenso, son **su superficie**. Un sistema con mil
validadores tiene mil veces más caro corromper el orden, y exactamente el
mismo tipo de poder concentrado en la función.

Este repositorio es inusualmente honesto en esto porque no puede
disimularlo: **tiene un solo ordenador de operaciones, y está escrito en
la primera línea de cada documento.** Un nodo único no es un consenso
peor; es el mismo intermediario, sin el maquillaje del número. Y tenerlo
a la vista permite hacer la pregunta buena, que no es *«¿esto está
descentralizado?»* sino **«¿qué haría falta para que esto no necesitara a
nadie?»**.

**¿Por qué existe el que ordena, para empezar?** Por una propiedad física
de la información clásica: **un bit se puede copiar.** El doble gasto es
un corolario de la clonabilidad, y el consenso es el parche que la
humanidad encontró para una limitación de la física, no para una del
dinero. Todo el edificio —bloques, quórums, finalidad, penalizaciones—
existe para decidir cuál de dos copias idénticas cuenta.

**El dinero cuántico ataca la raíz en vez del síntoma.** Un token
sostenido en un estado cuántico no se puede duplicar: no porque esté
prohibido, sino porque el **teorema de no-clonación** lo impide. Si el
objeto no es copiable, **no hay doble gasto que ordenar**, y el
intermediario de orden deja de tener función. Es la única propuesta
conocida que no reparte al último intermediario entre más manos, sino que
**lo suprime**.

**Y hay que decir lo que le falta, o esto sería propaganda.** No existe
hoy una construcción desplegable: exige memoria cuántica con coherencia
larga; el esquema original de Wiesner obliga a volver al emisor para
verificar —cambiando al ordenador por un verificador central, que no es
obviamente mejor—; y varias propuestas de dinero cuántico de clave
pública han sido rotas. **Es un horizonte, no un plan de obra.** Este
proyecto lo señala en `PRINCIPIOS.md` §6.bis y no afirma en ninguna parte
ser dinero cuántico: el README lo separa explícitamente.

**Entonces, ¿dónde se posiciona exactamente este proyecto?**

No como dinero cuántico, sino como **la aproximación clásica a la que ya
se le ha quitado todo lo demás**. Y eso tiene una consecuencia técnica
concreta, que es la tesis de este documento:

> **Si mañana llegara el dinero cuántico, casi nada de esta pila habría
> que rehacerlo.** La autoridad de gasto no es una firma que Shor rompa:
> es conocimiento de una preimagen. La solidez no descansa en curvas ni
> en emparejamientos: solo en funciones hash. El suministro no lo
> custodia la política de un cliente: está atado en el circuito y
> atestiguado en un registro encadenado. El cumplimiento no exige abrir
> el libro: es revelación acotada que el supervisor verifica sin acceso.
> **Lo único que habría que sustituir es al que ordena — y es justo lo
> que la física se llevaría.**

Esa es la posición, dicha en una frase: **este sistema no es
descentralizado, y su confianza residual, medida y escrita, se reduce a
dos residuos que son la misma sombra** —el orden y la completitud del
historial, y que el operador ve el estado (§2.bis)—. Ambos son el
ordenador de operaciones. Un sistema cuya confianza restante tiene
**nombre, medida y ataque diseñado** es un sistema que sabe qué le falta
para no necesitar a nadie; uno que se llama descentralizado y reparte al
ordenador entre miles, sabe menos de sí mismo.

Mientras el dinero cuántico no exista, quedan los cierres clásicos, todos
escritos y ninguno prometido como hecho: cabezas atestiguadas y acuse
(§121), anclaje externo de raíces (RFC-0012, `doc/ANCLAJE_EXTERNO.md`;
construido, pendiente de despliegue), consenso o
replicación. Cada uno **encarece** al último intermediario. Ninguno lo
elimina.

**Eliminarlo no es un problema de criptografía. Es un problema de
física** — y por eso vale la pena que exista, ya, una pila entera cuyo
único obstáculo restante sea ese.

---

### Fuentes externas citadas en este documento

Datos ajenos verificados el 06-08-2026; **re-verifícalos**, este campo se
mueve rápido:

- Fallo de sub-restringimiento en `halo2_gadgets` / pool Orchard de Zcash,
  divulgado en junio de 2026 (soft fork de emergencia el 2 de junio;
  NU6.2 el 3 de junio, bloque 3.364.600) y el esfuerzo de verificación
  formal del nuevo pool.
- Halo 2 y la eliminación de la ceremonia en Orchard (NU5, mayo de 2022);
  ceremonias previas de Sprout (2016) y Sapling (2018) — Electric Coin Co.
- Bitcoin: BIP-360 (Pay-to-Merkle-Root) fusionado el 11-02-2026; BIP-361
  («Post Quantum Migration and Legacy Signature Sunset»), 14-04-2026;
  estimaciones de 6-7 M BTC con clave pública expuesta.
- Ethereum: equipo de seguridad post-cuántica de la Fundación (enero de
  2026), `leanXMSS`/`leanVM`, hoja de ruta publicada en febrero de 2026 y
  horizonte declarado hacia 2029 — `pq.ethereum.org`.
- Google Quantum AI (marzo de 2026): estimación de ~1.200 qubits lógicos
  para romper curvas de 256 bits, ~20× menos que estimaciones previas.

---

*Refleja el estado conocido en el momento de escribirlo, sin auditoría
externa. Cuando este documento y el código dejen de coincidir, corrige el
documento.*
