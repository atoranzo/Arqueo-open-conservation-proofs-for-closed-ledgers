# ZK-SSL — Visión, principios de diseño y consecuencias técnicas

**Hacia una capa de liquidación como recipiente limpio**

*Revisión con lo construido y medido. Los principios no han cambiado; sí lo
que se sabe de sus consecuencias.*

---

## 1. Visión

Construir una capa de liquidación en la que:

- Lo esencial sea demostrable sin revelar lo innecesario.
- El poder del intermediario esté acotado, visible y mínimo.
- El sistema actúe como un **recipiente de reglas**: recibe operaciones,
  aplica invariantes y emite pruebas, sin apropiarse del contenido ni
  ocultar sus límites.
- La coherencia entre principio y arquitectura pese más que el
  rendimiento, la adopción prematura o la narrativa.

El sistema no aspira a eliminar toda confianza de un golpe. Aspira a **no
distorsionar lo real**: donde hay verificación, se verifica; donde aún hay
confianza, se nombra.

---

## 2. Principios de diseño

### P1 — No distorsionar lo real

Toda propiedad afirmada debe ser construible, medible y, preferiblemente,
refutable por test. Los errores propios se documentan. Las limitaciones se
declaran en primer plano, no en apéndices.

### P2 — Recipiente limpio

El sistema contiene estado y reglas. No debe ver más de lo necesario,
retener más de lo necesario ni depender de secretos de ceremonia
inauditables.

### P3 — Confianza mínima y explícita

Se elimina la fe donde la criptografía puede sustituirla. Se declara la fe
donde aún no puede eliminarse. Ninguna dependencia de confianza se vende
como soberanía.

### P4 — Medida (límites voluntarios)

El sistema define qué resuelve y qué rechaza conscientemente. Ampliar el
alcance no es un bien en sí. Preservar pureza y verificabilidad sí.

### P5 — Separación instrumento / operador / clave

Quien opera el nodo no debe necesitar las claves de gasto. Quien genera la
prueba no debe confundirse con quien la aplica. El diseño de la API debe
hacer difíciles los usos que reintroducen confianza innecesaria.

### P6 — Coherencia sobre brillo

Si una optimización introduce ceremonia, opacidad o superficie de fallo
difícil de auditar, se descarta aunque mejore latencia o tamaño de prueba.

### P7 — Vaciado y no acumulación

Los datos y privilegios existen el tiempo estrictamente necesario. El
sistema favorece el **descarte verificable** frente a la retención pasiva.

### P8 — Reproducibilidad sin protagonista

El valor del sistema reside en que terceros puedan reconstruirlo, medirlo y
atacarlo. La legitimidad no depende de la figura del autor.

---

## 3. Consecuencias técnicas

### 3.1 Sobre el estado y el operador

*Consecuencia de P2 y P3*

| Requisito | Implicación técnica | Estado |
|---|---|---|
| El operador no debe leer saldos en claro | Cifrado en reposo con clave que aporta al arrancar | ⚠️ **Parcial**: los ve en memoria |
| El operador no debe reescribir historial | Registro encadenado con verificación al arranque | ✅ |
| El operador no debe exigir claves de gasto | `account_view` + materiales → prueba en cliente → `apply` | ✅ |
| Censura y ordenación siguen siendo poder residual | Declaración en cabecera; consenso como único cierre | ⚠️ **Abierto** |

**Estado actual aceptado**: nodo único.
**Estado objetivo de coherencia**: minimizar la legibilidad del contenido
del vaso, no solo la falsificación del historial.

⚠️ **El cifrado en reposo no cierra P2.** Protege ante el robo del disco,
no ante el operador. Cerrarlo exigiría un rediseño por compromisos donde la
capa manipule solo raíces, y eso **no está hecho**.

### 3.2 Sobre el backend de pruebas

*Consecuencia de P3 y P6*

| Criterio | Decisión |
|---|---|
| Ceremonia de setup | **Prohibida** como dependencia soberana |
| Resistencia post-cuántica | Preferente |
| Rendimiento | Secundario respecto a la ausencia de ceremonia |
| Backend elegido | **STARK/FRI** (sin setup, post-cuántico) |
| Descartado pese a mejor rendimiento | Groth16 (pruebas 320× menores) |

**Regla de diseño**: no se adopta un paradigma que permita crear valor
indetectable mediante colusión en el setup.

⚠️ **Consecuencia medida y no anticipada**: la elección cierra la puerta a
la **agregación recursiva**. Winterfell no soporta recursión, y construirla
exigiría aritmetizar el verificador FRI. Ver §3.8.

⚠️ **La seguridad declarada es conjeturada.** Sin extensión de campo hay un
techo de 63 bits sobre Goldilocks. Con extensión de grado 2 el nivel es
razonable, y **cuesta 1,2× en tiempo y 1,7× en tamaño** — medido.

### 3.3 Sobre la API y el flujo de transferencia

*Consecuencia de P5*

Flujo obligatorio:

1. El cliente obtiene la vista pública y los materiales.
2. Calcula el nullifier y **genera la prueba localmente**.
3. La capa verifica y aplica **sin conocer la clave de gasto**.

La función de prueba es libre —no método de la capa— para que la API no
sugiera entregar la clave al nodo.

**Test discriminante**: `materials_alone_are_not_enough_to_spend`.

⚠️ **P5 tiene un límite en la autoridad de umbral**: en un nodo único,
quien genera la prueba de emisión necesita **ambas claves de custodio a la
vez**. La garantía real es *"dos claves comprometidas en lugar de una"*, no
*"dos voluntades independientes"*. Cerrarlo exige verificar firmas en
circuito, y **no está hecho**.

### 3.4 Sobre privacidad y cumplimiento

*Consecuencia de P1 y P3*

| Propiedad | Mecanismo |
|---|---|
| Privacidad frente a terceros que solo ven pruebas | Conocimiento cero sobre transferencias |
| Cumplimiento sin libro mayor | Revelación selectiva: exacto / mínimo / banda |
| Verificación externa | El supervisor verifica sin acceso al ledger |
| No fingir privacidad frente al operador | Declaración explícita |

⚠️ **FALLO GRAVE: el pagador ve el saldo del receptor.**

`TransferMaterials` entrega al cliente `receiver: AccountView`, que
contiene **el saldo exacto del receptor, su identidad pública y su nonce**.

Pagar un euro a alguien **revela cuánto tiene**.

| | El operador ve saldos | El pagador ve el del receptor |
|---|---|---|
| Quién | Una entidad, declarada | **Cualquiera que te pague** |
| ¿Estaba declarado? | Sí, en cabecera | **No** |

**Es inherente al modelo de cuentas.** La liquidación actualiza las dos
hojas, así que quien construye la prueba necesita el saldo del receptor
para calcular su hoja nueva. Zcash no lo tiene porque **no actualiza el
saldo del receptor**: crea una nota nueva.

**Cómo se dejó pasar.** El comentario del código decía: *"Son caminos de
Merkle y datos de cuenta: información de estado, no secretos"*. Esa frase
da por buena la fuga en vez de examinarla, y nadie escribió el test que la
habría delatado.

**Salidas, ninguna barata:**

| Vía | Coste |
|---|---|
| Protocolo entre dos partes: cada uno prueba su lado | Exige interacción y disponibilidad del receptor |
| Modelo de notas tipo UTXO | Rediseño del sistema entero |
| Que la capa construya la prueba | **Viola P5**: reintroduce la clave en el operador |

**Es más grave que la visibilidad del operador**, porque el operador es
**uno y está declarado**, mientras que una contraparte es **cualquiera**.

#### La corrección: transferencias por notas

**El pagador no actualiza el saldo del receptor: crea una nota para él**, y
el receptor la reclama.

```text
nota = H(H(identidad_receptor, r), [importe, época])
```

| | Hoy | Con notas |
|---|---|---|
| El pagador necesita | Saldo, identidad y nonce del receptor | **Solo su identidad pública** |
| Lo que aprende del receptor | **Cuánto tiene** | **Nada** |

**Qué aprende cada parte, verificado:**

| Parte | Aprende |
|---|---|
| Pagador | La identidad pública del receptor, que usa como dirección |
| Receptor | Nada del pagador |
| Un tercero | Nada: solo ve compromisos |

**Y el pagador no puede gastarla.** Conocer `r` no basta: reclamarla exige
demostrar la **clave de gasto** del receptor.

**Las piezas ya existen** en el proyecto derivado: `circuit_issue` crea una
nota atada a una identidad, y `circuit_redeem` la reclama demostrando que
es tuya. Fueron escritas para el modo sin conexión y **resuelven este
problema sin cambios**.

#### ⚠️ El residuo que deja

**El pagador elige `r`, así que reconoce la nota cuando se reclama.** Sabe
**cuándo** cobra el receptor, aunque no cuánto tiene.

Zcash lo cierra cifrando la nota para que el receptor derive `r`. **Aquí no
está resuelto**, y es mucho menor que revelar el saldo — pero es una fuga
de vinculabilidad y conviene nombrarla.

#### ⚠️ El coste de usabilidad

**La transferencia pasa a ser en dos pasos**: el pagador crea la nota, el
receptor la reclama. El dinero queda pendiente hasta que el receptor actúe.

Es exactamente el modelo de Zcash, y no es gratis: hoy un pago se completa
solo, y con notas **el receptor tiene que hacer algo**.

**Estado: diseño analizado, sin implementar.** El refactor de la capa
—transferencias en dos fases, notas pendientes, reclamación— se estima en
15-25 rondas.

⚠️ **Sin revelación forzosa.** Si el titular no coopera, no hay mecanismo
alternativo. Es deliberado —no existe clave maestra que robar— y en un
despliegue regulado sería una decisión de política a evaluar.

⚠️ **Fugas por canal lateral: evidencia débil.** El tamaño de prueba varía
un 5,4% y la correlación con el importe es +0,008 en escala logarítmica. El
tiempo tampoco depende del importe: dispersión de 1,19×. **Con 16 y 4
muestras eso no es demostración.**

### 3.5 Sobre límites, cierre y moderación

*Consecuencia de P4 y P7*

| Mecanismo | Propósito | Estado |
|---|---|---|
| Tope de emisión inmutable | Límite duro de creación de valor | ✅ |
| Contadores públicos de intervención | Visibilidad del uso de privilegios | ✅ |
| Caducidad de privilegios | Evitar poder residual permanente | ✅ **congelaciones** |
| Rechazo explícito de alcance | Documentar qué no se construirá | ✅ |
| Instantáneas deterministas y verificables | Copia sin formatos opacos | ✅ |
| **Descarte verificable frente a retención** | Reducir superficie sensible | ✅ **ver §3.8** |

### 3.6 Sobre verificación y método

*Consecuencia de P1 y P8*

| Práctica | Requisito técnico |
|---|---|
| Cada propiedad de seguridad | **Test discriminante**: testigo coherente que viola solo esa restricción |
| Mediciones | Misma máquina, modo release, cifras propias |
| Errores de diseño o medición | **Documentados, no borrados** |
| Dependencias | Minimizar; evitar toolchains ocultas |
| Auditoría externa | Necesaria; los tests propios **no la sustituyen** |

**Práctica añadida por la experiencia — el test que valida al test.**

Varias propiedades llevan un segundo test que comprueba que el primero
**puede fallar**:

| Test | Su validador |
|---|---|
| Los saldos no son legibles en disco | Sin cifrado **sí lo son** |
| Una cuenta congelada no puede gastar | Una libre **sí puede** |
| Dos gastos revelan la identidad | Un gasto **no revela nada** |
| El doble gasto entre lotes se detecta | Sin registro **no se detecta** |

Sin el segundo de cada par, el primero pasaría aunque la comprobación
estuviera mal construida.

**⚠️ Práctica que faltaba y ahora consta: los hallazgos deben vivir en el
código.**

Dos de los ocho hallazgos documentados se **reintrodujeron** al escribir
código nuevo sin consultar lo aprendido: la identidad de 64 bits y el techo
de solidez. Documentar no impide repetir. Un test que falle, sí.

### 3.7 Sobre gobernanza y emisión

*Consecuencia de P3, P5 y P8*

| Elemento | Diseño | Estado |
|---|---|---|
| Emisión | Umbral 2-de-N | ✅ |
| Gobernanza de custodios | 2-de-N con contador público | ✅ |
| Inmutabilidad de límites y tope | Sí | ✅ |
| **Avance del tiempo** | **2-de-N, de una en una** | ✅ |
| Sustituibilidad | Evitar roles cuya pérdida congele el sistema | ⚠️ **Parcial** |
| Legitimidad | Reproducibilidad > autoridad del autor | ✅ |

**El tiempo también es un privilegio.** El sistema necesitó una noción de
época para los plazos, y avanzarla es un poder: quien la controle **caduca
congelaciones antes de tiempo**. Se cierra igual que los demás —umbral y
contador— y **sin saltos**, para que ningún avance masivo pase inadvertido.

⚠️ **El conjunto de gobernanza es inmutable.** Si se compromete, la única
salida es un ledger nuevo. Es el final consciente de la cadena de
autoridad, y contradice parcialmente el criterio de sustituibilidad.

### 3.8 Sobre la retención de pruebas — **P7 aplicado**

*Consecuencia de P7, y el caso que más lo justifica*

**El problema.** Mil transferencias acumulan 59 MB de pruebas. A un millón
de operaciones diarias son **59 GB al día**. Eso hace indesplegable el
sistema.

**La respuesta habitual —agregar— no es alcanzable.** La recursión exigiría
aritmetizar el verificador FRI. Y agrupar por lotes exigiría que alguien
tuviera **las N claves de gasto** a la vez, lo que viola P5 directamente.

**La pregunta que faltaba**: ¿quién necesita la prueba después de
aplicarla?

| Parte | ¿La necesita? |
|---|---|
| El operador | **No.** Ya la verificó |
| El titular | **Sí**, si algún día quiere que un tercero re-verifique la suya |

**Se asumía retención central sin examinarla.** Y P7 dice lo contrario:
*los datos existen el tiempo estrictamente necesario*.

| | Retención central | Retención distribuida |
|---|---|---|
| Operador (1M op/día) | 59 GB/día | **131 MB/día** |
| Titular (10 op/mes) | — | 620 KB/mes |

**463 veces menos.** El operador guarda la entrada del registro encadenado
—137 bytes— en vez de la prueba.

**Y el descarte es verificable, no pérdida**: el registro ya guarda
`H(prueba)`. Quien conserve una copia demuestra que es exactamente la que
el registro dice, sin que el operador la tenga.

⚠️ **Lo que sigue abierto**:

- Si nadie conservó una prueba, esa transición **no puede re-verificarse**.
  El registro muestra la cadena, no la validez criptográfica de esa
  operación.
- El titular **carga con la custodia**. En un despliegue real la asumiría
  su proveedor, con lo que la retención se re-concentra —repartida entre
  proveedores, no en el operador central—.
- **La verificación en bloque sigue necesitando N verificaciones.** Eso
  *sí* exigiría recursión, y es un problema menor que 59 GB diarios.

**Lección metodológica.** Una versión anterior de este análisis concluía
que el límite de escala era *"la razón principal por la que esto no es
desplegable"*. Era una conclusión precipitada, apoyada en una suposición
—retención central— que **nunca se examinó y que este documento ya
contradecía**.

El principio estaba escrito antes que el problema. Aplicarlo tardó tres
rondas más de lo necesario.

### 3.9 Sobre la legibilidad del estado — **P2, diseño demostrado**

*Consecuencia de P2. La mayor incoherencia declarada del sistema.*

**El problema.** La capa guarda `(identidad, saldo, nonce)` de cada cuenta
y **el operador los ve en memoria**. El cifrado en reposo protege ante el
robo del disco, no ante él. P2 —*no debe ver más de lo necesario*— **no se
cumple**.

**La observación.** La capa no necesita ese contenido. Para mantener el
árbol y verificar transiciones le basta con **el digest de la hoja**.

| | Hoy | Por compromisos |
|---|---|---|
| La capa guarda | id, saldo, nonce | **Solo `H(H(id,saldo),nonce)`** |
| Calcula raíces y caminos | Sí | **Sí** |
| **Puede leer un saldo** | **Sí** ⚠️ | **No** |

**Por qué sigue siendo sólido.** El cliente aporta la posición y el digest
de la hoja nueva. La capa verifica la prueba, coloca la hoja, y **comprueba
que la raíz resultante es la que la prueba acredita**. Si el cliente
mintiera sobre la hoja o la posición, esa comprobación falla.

**No necesita entender el contenido para detectarlo.**

**Estado**: demostrado en `commitment.rs` con **7 tests**, incluidos los dos
ataques —mentir sobre la hoja, mentir sobre la posición— y la comprobación
de que la capa no retiene ningún saldo, con su validador.

**Y al implementarlo cambió el planteamiento.** Era un refactor de 101
referencias; es **una migración**:

| | |
|---|---|
| La vía nueva (`send`/`claim`) | **No lee el registro en ningún punto** |
| La vía antigua (`transfer`, emisión…) | Lo necesita, y está marcada |
| Cuando no quede vía antigua | El registro se borra |

El saldo lo aporta el titular y la capa **comprueba que produce la hoja que
tiene en el árbol**. Si mintiera, no coincidiría: probado en
`a_holder_lying_about_their_balance_is_caught`, con su validador.

La ventaja sobre el refactor de golpe es que **en cada momento el sistema
funciona**. Y el criterio de aceptación de §5 lo aprueba: reduce lo que el
operador ve, declara lo que queda, y no rompe lo que había.

#### Estado de la migración

| Módulo | Lecturas del saldo |
|---|---|
| `two_phase` (`send`/`claim`) | ✅ **0** |
| `burn` | ✅ **0** |
| `audit` | ✅ **0** |
| `mint` | ⚠️ 2 |
| `recovery` | ⚠️ 2 |
| `transfer` (antigua, con la fuga) | ⚠️ 4 |
| `client` (materiales) | ⚠️ 3 |
| `accounts` (los propios getters) | ⚠️ 3 |

⚠️ **Una versión anterior de esta tabla decía que `mint`, `recovery` y
`transfer` estaban a cero. Era falso**: la comprobación buscaba
`self.records.get` en una línea, y esos módulos lo escriben en varias.

Es el mismo fallo que aparece en §8: **tomar el resultado de una búsqueda
como evidencia sin verificar que la búsqueda era completa**. Aquí produjo
una tabla incorrecta en un documento público.

**`disclose_exact` era el caso más claro.** Leía el saldo con `balance_of`
—*"pregúntale a la capa cuánto tienes y demuéstralo"*— que es justo lo que
este modelo elimina. Ahora es *"demuestra lo que dices tener"*.

#### Las dos operaciones de custodios no se migran igual

**`mint`**: los custodios acreditan una cuenta ajena, así que necesitan su
saldo para calcular la hoja nueva. Es **el mismo problema que la
transferencia antigua**, y tiene la misma solución: **emitir a un pendiente
que el titular reclama**.

**Implementado** en `circuit_mint_pending`: **16 tests**. Sube el
suministro y crea un pendiente **sin tocar ninguna cuenta**, así que los
custodios no necesitan el saldo de nadie.

La propiedad va en el tipo: la firma recibe la identidad del receptor y un
aleatorio. **No hay parámetro donde entrara un saldo, ni columna donde
alojarlo.**

⚠️ **Un fallo encontrado al construirlo, y sin corregir**: el tope de
emisión se transporta y se declara públicamente, pero **falta la
comprobación de rango**. Es una restricción que existe en el nombre y no
impone nada — el mismo modo de fallo que este documento describe en otros
sitios.

Apareció al preguntar *"¿cada columna que declaro se usa de verdad?"*, que
además destapó que **siete columnas nuevas nunca se rellenaban**: valían
cero y sus restricciones se cumplían trivialmente.

Cerrar el tope exige un segmento de rango más, lo que cambia
`NUM_SEGMENTS` y con él los índices de las periódicas. **No está hecho.**

**Cableado a la capa**: `mint_to_pending` y `apply_mint_to_pending`, con
**4 tests de integración**. El ciclo completo funciona: los custodios
emiten a un pendiente y el destinatario lo reclama con `claim`.

⚠️ **`mint()` clásico sigue existiendo.** Retirarlo exige migrar lo que lo
use, y **no está hecho**.

⚠️ **El tope lo impone la capa, no el circuito.** El test se llama
`the_supply_cap_is_enforced_by_the_layer` precisamente para que se vea. Es
coherente con el modelo declarado —el operador ya controla la capa— pero va
contra el principio de imponer en el circuito.

**`recovery`**: aquí hay una **tensión estructural**, no solo trabajo
pendiente.

Los custodios cambian la identidad de una cuenta conservando su saldo. Para
calcular la hoja nueva hace falta ese saldo, y:

| Quién podría aportarlo | Problema |
|---|---|
| El operador | Es justo lo que este modelo elimina |
| Los custodios | No lo conocen |
| **El titular** | **Perdió la clave — puede haber perdido también sus registros** |

Si el titular conserva sus anotaciones aunque haya perdido la clave, puede
aportarlo y la migración es trivial. **Si perdió el dispositivo entero, no
lo sabe, y la recuperación se vuelve imposible.**

> **La recuperación por custodios y un operador que no ve saldos son
> parcialmente incompatibles.** Se puede tener una u otra en su forma
> plena, no las dos.

Cerrarlo del todo exigiría que el saldo estuviera custodiado por umbral —el
mismo diseño que la identidad abrible del pago sin conexión— y **eso da a
los custodios un poder nuevo que habría que contar y acotar**.

**No está resuelto, y no es evidente que deba resolverse a favor de la
privacidad**: una recuperación imposible es un fallo peor que un operador
que ve saldos.

⚠️ **El operador sigue viendo los saldos** mientras queden operaciones sin
migrar. La migración avanza una a una, y **no está terminada**.

⚠️ **El coste, que es real.** El titular lleva su propio estado. Si pierde
su copia local **no sabe cuánto tiene**, aunque el dinero siga ahí y la
recuperación por custodios funcione.

Hoy la capa puede responder *"tienes X"*. Con este diseño **no puede**. Es
el modelo de Zcash y un cambio de usabilidad serio. Un despliegue real lo
resolvería con el proveedor de pago guardando el estado del cliente — con
lo que la legibilidad se re-concentra, repartida y no en el operador
central.

### 3.10 Sobre la rotación de privilegios

*Consecuencia de P4 y P7: los privilegios existen el tiempo estrictamente
necesario.*

**El problema**: sin rotación, una clave de custodio comprometida **sirve
para siempre**. Los contadores hacían visible su uso; nada lo acotaba.

**El diseño**: la rotación se expresa **por uso, no por tiempo**, porque
esta capa no tiene noción de tiempo.

| | |
|---|---|
| Emitir, congelar, recuperar | Consumen una intervención |
| Al agotarse el cupo | Los custodios **dejan de poder actuar** |
| Rotar el conjunto | **Reinicia** el contador |

Agotarse **no bloquea el sistema: obliga a renovar**. Y el conjunto viejo
queda inerte por otra vía — su raíz ya no es la vigente.

**5 tests**, incluido el que comprueba que el cupo **sobrevive al
reinicio**: si no, bastaría reiniciar para seguir usando un conjunto
agotado.

**Dos decisiones que el compilador y el orden delataron:**

El consumo va **al aplicar**, no al generar la prueba. Puesto en la
generación, pruebas descartadas habrían agotado el cupo. El compilador lo
señaló al rechazar una mutación sobre `&self`.

Y va **después** de verificar la autoridad: antes, cualquiera podría agotar
el cupo de los custodios sin serlo.

⚠️ **Lo que NO cubre.** `set_max_custodian_uses` no está protegido por
ninguna autorización: **un operador puede subir el cupo y anular la
rotación**.

Es coherente con el modelo declarado —el operador ya controla la capa— pero
la rotación es **una política que el operador aplica, no una garantía que
le vincule**. Imponerla exigiría llevar el cupo a los circuitos de emisión,
congelación y recuperación. **No está hecho.**

---

## 4. Orden de prioridad para mayor coherencia

*Actualizado con lo hecho.*

| | Prioridad | Estado |
|---|---|---|
| ~~**0**~~ | ~~Cerrar la fuga del saldo del receptor al pagador~~ | ✅ **RESUELTO**: `send`/`claim` en dos fases, 38 tests. La vía antigua sigue existiendo (§3.4) |
| 1 | Reducir la legibilidad del estado por el operador | ⚙️ **En migración**: la vía nueva (`send`/`claim`) **no lee el registro**. La antigua sí, y está marcada (§3.9) |
| 2 | Mantener y endurecer la separación clave / capa | ✅ Salvo la autoridad de umbral (§3.3) |
| 3 | Formalizar el catálogo de rechazos | ✅ Hoja de ruta con lo no alcanzable y lo innecesario |
| 4 | Privilegios con medida: rotación, contadores, caducidad | ✅ **Completa**: contadores, caducidad de congelaciones y rotación por uso (§3.10) |
| 5 | Consenso distribuido | ⬜ **Abierto**. Único cierre real de censura |
| 6 | Auditoría externa | ⬜ **Condición, no capacidad** |
| 7 | ⚠️ **Capacidad del árbol de nullifiers** | ⬜ **Abierto**: ~65.000 pagos, no 2³². Ver `AUDITORIA.md` §13 |
| 8 | ⚠️ **Agotamiento del árbol de pendientes** | ⚙️ **Declarado**: falla con su causa, pero el límite sigue (§13) |

**La prioridad 0 apareció al empezar el refactor de la 1**, y la desplaza.
Cerrar la visibilidad del operador mientras cualquier contraparte ve los
saldos sería resolver el problema menor primero.

La prioridad 1 ya no es una incógnita: el diseño está demostrado (§3.9) y
lo que falta es trabajo mecánico.

---

## 5. Criterio de aceptación de cambios futuros

Todo cambio de arquitectura, backend o API debe responder afirmativamente
a:

1. ¿Preserva la ausencia de ceremonia inauditable?
2. ¿Reduce o declara la confianza residual?
3. ¿Evita que el operador **vea o retenga** más de lo necesario?
4. ¿Mantiene tests discriminantes para las propiedades afectadas?
5. ¿Puede explicarse sin vender como resuelto lo que sigue abierto?

**Criterio añadido por la experiencia:**

6. ¿El hallazgo que motiva el cambio queda **en el código**, no solo en un
   documento?

Si alguna respuesta es negativa, el cambio no es coherente con la visión.

### 5.1 Aplicación registrada del criterio

| Cambio evaluado | Respuesta | Resultado |
|---|---|---|
| Agrupar pruebas por lotes | Falla el 3: exigiría las claves de gasto | **Rechazado** |
| Delegación de prueba | Innecesario: 620 ms y 14 MB caben en un móvil | **No construido** |
| Retención distribuida | Cumple los seis | **Adoptado** |

Rechazar por criterio y **documentar el rechazo** es lo que P4 llama
*catálogo de rechazos*.

---

## 6. Proyecto derivado

Los circuitos de ZK-SSL se reutilizan en
**[euro-digital-zk](https://github.com/atoranzo/euro-digital-zk)**, que
implementa requisitos concretos del reglamento del euro digital: límite de
tenencia demostrable, desbordamiento a cuenta vinculada, pago sin conexión
con revelación de identidad por doble gasto, y devolución de billetes no
gastados.

Sirve de **prueba de los principios**: al aplicarlos a requisitos ajenos y
publicados, se ve cuáles resisten. Los ocho resistieron; dos hallazgos se
reintrodujeron por no consultarlos (§3.6).

---

## 7. Cierre

Este documento fija la visión de ZK-SSL como **recipiente de liquidación
con reglas demostrables**: mínima fe, máxima claridad de límites,
separación entre instrumento y poder residual, y preferencia por la pureza
verificable sobre el brillo.

La arquitectura no se evalúa solo por lo que permite hacer. Se evalúa por
**lo que impide falsear** y por **lo que se atreve a declarar que aún no
resuelve**.

Y hay dos pruebas de que los principios funcionan, ninguna prevista.

**P7 resolvió un problema que parecía requerir una tecnología que este
proyecto no tiene.** El principio precedió al problema, y bastó con
aplicarlo.

**P2 destapó un fallo peor que el que iba a arreglar.** Al empezar el
refactor para que el operador no viera los saldos, apareció que **cualquier
contraparte ya los ve** (§3.4). Llevaba ahí desde el principio, con un
comentario que lo justificaba en vez de examinarlo.

Ese es el argumento a favor de este documento: **un principio aplicado en
serio encuentra lo que la implementación daba por bueno**.

---

*Angel Toranzo Portela · MIT / Apache-2.0*
