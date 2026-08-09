# Política de seguridad — ZK-SSL

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
- **Nodo único operado por una sola parte.** Pese a la palabra «Sovereign»,
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
  ⚠️ Garantía condicional, y **hoy nadie fuera del operador observa cabezas**
  (§2).
- **Privacidad del contenido frente a terceros**: ⚠️ **medido que no se
  cumple**, ver §3.2.

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
  (`circuit_send`: 23 clases · 1288 celdas-clase · **0 sin dueño**;
  `circuit_claim`: 21 · 1155 · 0). Eso cierra una pregunta concreta —«¿hay
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
- **El operador puede censurar y ordenar.**
- **La custodia del registro está en manos del operador.** Sin observadores
  externos de sus cabezas, podría presentar historias distintas a partes
  distintas. **Desde §241-§243 el nodo firma, emite y sirve sus cabezas, y
  existe un verificador independiente** (`zk-ssl-verify`) — pero **no hay
  ningún testigo**, así que la propiedad sigue sin cerrarse.

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
- **No hay recuperación si el nodo desaparece.**
- **Metadatos**: qué posiciones cambian y cuándo siguen siendo observables.
  Medido campo a campo en §231: un envío revela **emisor, importe y
  `notice.position`** —no el receptor—; un cobro revela **receptor,
  importe y la misma posición**. Quien vea las dos mitades reconstruye la
  arista por esa clave. El nodo las ve siempre; un **agregador** (§223)
  solo si procesa ambas, así que separarlas entre agregadores distintos
  es una mitigación real.
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
   raíces (interfaz diseñada: `doc/ANCLAJE_EXTERNO.md`; pendiente de despliegue).

2. **El operador ve el estado.** La privacidad es frente a terceros,
   no frente a quien mantiene el ledger; el titular tiene vista
   autenticada (49-A) y el resto es asumido y documentado. Eliminación:
   arquitectura de operador ciego (B11) o federación.

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
| `zkssl_publicId`, `zkssl_logEntries`, `zkssl_epochHead`, `zkssl_supply`, `zkssl_accountCount` | ninguna | **público por diseño** — el registro y los agregados son auditables a propósito |
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
un reenvío presenta una raíz obsoleta y se rechaza.

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

## 3.bis La superficie de protocolo (§197-§201): qué añade y qué defiende

Desde agosto de 2026 esto no es solo una capa: hay cable, nodo, SDK y un
contrato público. Una superficie nueva es, por definición, riesgo nuevo —
y también trae dos defensas que antes no existían.

**Riesgo que añade** (arriba, §3.3): un puerto abierto sin auth ni TLS.

**Lo que el diseño protege, verificado en código:**

- **La clave de gasto no viaja, y no *puede* viajar.** El pago va en dos
  fases y la prueba se genera **en local** (`prove_send`/`prove_claim`);
  `Wallet::spend_key` es privado y **ni siquiera implementa
  `Serialize`** — no hay accidente posible por serialización.
- **El cable rechaza lo que no entiende**: DTOs con
  `deny_unknown_fields`, hex canónico, digests de anchura fija.
- **Los vectores de conformidad son una defensa, no solo documentación.**
  Una segunda implementación que divergiera —en la hoja, en el orden, en
  el encadenamiento— **no pasaría `conformance --check`**: la divergencia
  se vuelve detectable campo a campo en vez de silenciosa.
- **El wallet en reposo** (§199) usa la misma construcción que el ledger
  con **dominio propio**, y un test exige que la clave del ledger **no**
  abra el keystore. ⚠️ Su KDF es SHA-256, que **no** es una función de
  derivación de contraseñas: una contraseña débil es forzable. Está
  documentado en el módulo y el endurecimiento a Argon2 tiene cauce
  abierto (RFC-0001, `spec/rfc/`) — **no es un descuido, es una deuda con
  expediente**.

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
| **ZK-SSL** | **conocimiento de preimagen**: identidad, salt de hoja y autoridad derivan de la clave **por hash** (§117) | **no hay firma clásica en la vía de pago**, y STARK/FRI solo usa hashes: no hay curva que romper |

⚠️ **La reserva que toca hacerse**: «post-cuántico» aquí significa *sin
supuestos de curva*, no *invulnerable*. Grover degrada los hashes; y este
proyecto **midió y publicó** que su configuración por defecto tiene techo
de **63 bits de solidez** sin extensión de campo (hallazgo 3), frente a
los ~128 conjeturados que se suelen citar. Un sistema con miles de
validadores y años de producción sigue siendo, hoy, **más seguro en la
práctica** que uno sin auditar.

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

| eje | ellos | ZK-SSL |
|---|---|---|
| **Descentralización** | miles de validadores/mineros independientes | **UN nodo, un operador**. Ve el estado, ordena, puede censurar |
| **Rendimiento** | Solana en miles de TPS; Bitcoin y Ethereum en un orden muy superior a este | **1,5-1,9 TPS** medidos (§123) |
| **Madurez** | años en producción, auditorías repetidas, recompensas por fallos | **cero auditorías externas**, prototipo de investigación |
| **Tamaño de prueba** | Groth16: 192 B | **53,6-65,3 KB** medidos en los circuitos de esta capa (§218) — el precio de no depender de nadie. Los 36,7 KB de las tablas comparativas son del circuito de comparación, no de éstos |

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
(§121), anclaje externo de raíces (`doc/ANCLAJE_EXTERNO.md`), consenso o
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
