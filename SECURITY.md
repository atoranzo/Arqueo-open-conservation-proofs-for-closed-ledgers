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
  `tools/banco_mentiroso_sin_resolver.sh --largo`).
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
delante es exactamente el escenario que este apartado advierte.**

⚠️ **Y un segundo hallazgo, nuevo**: el nodo abre el ledger **sin cifrado
en reposo**. La capa tiene `open_encrypted` y `zk_ssl::crypto` desde hace
mucho, y el wallet del SDK duerme cifrado desde §199 — **pero el binario
del nodo no cablea ninguna clave**. Quien robe el disco de un nodo lee
los saldos. No es un fallo de la primitiva: es que no está conectada.

**Estado: parcialmente resuelto (vista autenticada), y abierto en
despliegue** (sin auth/TLS/límite de tasa; ledger del nodo en claro).
Backlog 49. `AUDITORIA.md` §93.1, §129, §157.

### 3.4 Espacio de los identificadores anti-doble-gasto — **no aplica**

La vía de producción es la de **dos fases**, y **no usa nulificadores**: un
envío cambia el saldo del pagador, luego su hoja, luego la raíz, de modo que
un reenvío presenta una raíz obsoleta y se rechaza. Y la raíz sola no bastaba:
el estado global se repite tras un reembolso o un ciclo A→B→A, y un recibo ya
aplicado volvía a valer. Desde el §654 la capa guarda además la huella de cada
prueba del titular ya aplicada y rechaza la que vuelva, y desde el §659 también
la que sólo cambie en nodos de más dentro de un lote de Merkle (§3.9).

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
`arqueo-verify-v0.4.0` (§658) los rechaza, medido con el kit descargado.

### 3.9 Ocho hallazgos de un análisis con agentes — ⚠️ MEDIDO, siete cerrados y uno acotado

En el §637 una sesión de Claude Code analizó el árbol con un enjambre de agentes
([`doc/blueprint-v2.md`](doc/blueprint-v2.md)) y entregó al autor, en privado, ocho hallazgos
abiertos en `main`. Se corrigieron antes de describirlos aquí, como pide la sección 5. Tres se
reprodujeron con tests antes del arreglo; cada arreglo lleva falsadores que caen con él desactivado.

| hallazgo | qué rompía | prioridad | cerrado en |
|---|---|---|---|
| El hexadecimal del cable y del kit se troceaba por bytes de un `&str` | una sola petición sin credencial con un carácter multibyte paraba el nodo (PARADA); el kit salía con 101 en vez de ROJO; `+` y mayúsculas se aceptaban | P0 | §650; ⚠️ en los `u64` (QUANTITY), el §662 |
| La guarda de forma no miraba la marca del meta | una prueba oculta con el meta vaciado hacía entrar en pánico a la capa (con el candado del nodo tomado) y al kit | P0 | §651 |
| Las semillas de la ocultación eran de 64 bits | quien ve una prueba despejaba cualquier columna constante del testigo —la clave de gasto, la de un custodio— con unas 2^74 compresiones Blake3 (ESTIMADO) | P0 | §652 |
| `applySend` y `applyClaim` guardaban el importe y el saldo que mandaba el cliente | un titular rompía la conservación en la contabilidad `u64` y el libro no volvía a abrir | P0 | §641 (RFC-0017) |
| `Proof::from_bytes` aceptaba bytes de cola | la huella de una prueba no identificaba una operación | P1 | §653 |
| El reenvío sólo se paraba por la igualdad de raíces | tras un reembolso o un ciclo A→B→A, un recibo ya aplicado volvía a valer | P1 | §654 |
| `zkssl_supply` publica sin credencial el total en tránsito | por diferencias da el importe de cada envío y cobro | P1 | **acotado**: ver abajo |
| Un fallo del almacén no paraba el nodo | la memoria podía firmar raíces que el disco no tiene | P2 | §655 (c) |

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
La versión nueva del kit está publicada: `arqueo-verify-v0.4.0` (§658); la `v0.3.0` sigue afectada
por los dos primeros y por la cola. ⚠️ **Y una variante de la cola, por dentro (§659)**: nodos de más
dentro de un lote de Merkle no movían la raíz, así que la prueba verificaba con otros bytes y el
índice de pruebas aplicadas del §654 se esquivaba; la capa y el kit los rechazan desde el §659, y la
vigente era `arqueo-verify-v0.4.1` (§661); hoy es `arqueo-verify-v0.4.2` (§671), que corrige además los §662 a §664. `AUDITORIA.md` §637, §641,
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
- **El nodo no constituye libros.** Con la feature `dev`, la de la compilación por defecto,
  abre la capa con las raíces de la suite, de claves de un elemento escritas en
  `crates/zk-ssl/src/tests_support.rs`; sin ella no arranca. `--custodian-root` y
  `--governance-root` siguen pendientes.
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

Lo fijan tests desde el §697: en las familias de longitud fija, conjeturada ≥ 127, LDR ≥ 80 y
UDR = 59 sobre pruebas ocultas reales; en la edad, el piso de cada `m`. **El nivel que se
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
dobla —112,8 con 2.048, y 96,8 en la edad con `m = 23`—, y, puesto en lugar del campo, es el menor
de los términos de la fórmula: las consultas dan 189 bits y el hash pone 128. Con él, la
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
que costaría llevar las de producción a 128 bits demostrables no está medido.

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
  tiene la clave derivada en memoria.

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
extensión de campo», y no decía que el nivel cuántico no está medido.

**2. Sin ceremonia de confianza — y sin haberla tenido nunca.** Zcash la
eliminó con Halo 2 en Orchard (mayo 2022), pero Sprout y Sapling nacieron
de ceremonias y esos pools legados existen. Aquí el arranque **no genera
claves**: 0,67 ms medidos, no hay secreto que destruir ni participante en
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
| **Tamaño de prueba** | Groth16: 192 B | **72.382-84.244 B** por prueba de envío o de cobro, ocultas, la banda que ata un test desde el §538 (un pago son dos: 145.953-167.967 B); el tamaño de las demás pruebas no lo ata ninguna banda — el precio de no depender de nadie. Antes de ocultar, el envío y el cobro medían 64,6 y 65,3 KB (§218). Los 36,7 KB de las tablas comparativas son del circuito de comparación, con otras opciones y sin ocultar |

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
el borrador del aviso, nunca en una rama, un PR o un *issue* públicos— y se
publica de una vez: el código, los vectores, el RFC o la enmienda con su etapa
ya construida, el asiento de `AUDITORIA.md`, la release y el aviso. Los vectores
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
