# Preparación para auditoría externa

**Este documento lo escribe quien programó el sistema.** No es una
auditoría: es el material para que otra persona la haga con eficacia, más
una lista honesta de los puntos donde el autor tiene menos confianza.

La ceguera del autor es precisamente lo que una auditoría corrige. Todo
lo que sigue debe tratarse como **afirmaciones a verificar**, no como
hechos establecidos.

---

## 1. Qué auditar, y qué no

### Merece auditoría

| Componente | Por qué |
|---|---|
| `stark-experiment/src/circuit_*.rs` | Los seis circuitos. Un fallo aquí rompe garantías |
| `zk-ssl/src/*.rs` | La capa: encadenamiento, persistencia, precondiciones |
| `stark-experiment/src/rescue_hash.rs` | Las constantes y el gadget de hash |

### No merece auditoría (todavía)

`zk-core`, `halo2-experiment`, `plonk-experiment`, `nova-experiment` y
`ceremony` son **trabajo comparativo**, no la capa. No están en el camino
de ejecución del sistema.

---

## 2. Modelo de amenaza

### Adversarios contemplados

| Adversario | Qué puede | Qué NO debe poder |
|---|---|---|
| **Titular de cuenta** | Firmar con su clave | Gastar más de su saldo, gastar dos veces, gastar de otro |
| **Tercero sin claves** | Ver pruebas, enviar mensajes | Nada que altere el estado |
| **Custodio único** | Firmar como custodio | Emitir, recuperar cuentas |
| **Dos custodios** | Emitir, recuperar | Superar el tope, cambiarse a sí mismos |
| **Un gobernador** | Firmar como gobernador | Cambiar el conjunto de custodios |

### Adversario NO contemplado, y es el mayor

**El operador del nodo.** Ve todos los saldos, ordena las operaciones,
puede censurar, y es un punto único de fallo. Está documentado en
cabecera del README y no se ha intentado mitigar: requiere consenso
distribuido.

**Un auditor no debería perder tiempo buscando ataques del operador: son
triviales y conocidos.**

### Adversario NO contemplado: dos gobernadores comprometidos

Es el final consciente de la cadena de autoridad. No hay salida salvo
crear un ledger nuevo.

---

## 3. Invariantes de seguridad y dónde se imponen

Cada una debería verificarse **en el circuito**, no solo en la capa. Las
comprobaciones de la capa son comodidad; la garantía está en el AIR.

| Invariante | Circuito | Restricción |
|---|---|---|
| Conservación del dinero | `circuit_settlement` | `C_CONSERVATION` |
| Solo el titular gasta | `circuit_settlement` | `C_PK_CHECK` |
| Nadie gasta dos veces | `circuit_settlement` | `C_NULL_EMPTY` + raíces públicas |
| `importe <= saldo` | `circuit_settlement` | Rango sobre `saldo − importe` |
| `importe <= límite` | `circuit_settlement` | Rango sobre `límite − importe` |
| Solo 2 custodios emiten | `circuit_mint` | Aserciones sobre `custodian_set_root` |
| Un custodio no cuenta dos veces | `circuit_mint`, `circuit_threshold` | Rango sobre `idx_b − idx_a − 1` |
| El índice no se puede mentir | ídem | `C_ACC` + `C_ACC_FINAL` |
| El suministro refleja la emisión | `circuit_mint` | `C_SUPPLY` |
| No se supera el tope | `circuit_mint` | Rango sobre `tope − suministro` |
| Destruir no crea dinero | `circuit_burn` | `C_BALANCE`, `C_SUPPLY` |
| Recuperar no mueve dinero | `circuit_recovery` | `C_INPUT` (ambos carriles, misma columna) |
| Las recuperaciones son contables | `circuit_recovery` | `C_COUNT` |
| Un custodio no gobierna | `circuit_governance` | Dominio separado + raíz de gobernanza |
| Revelar exige ser titular | `circuit_audit` | `C_PK_CHECK` |
| No se finge solvencia | `circuit_audit` | Rango sobre `saldo − inferior` |

**Impuesto solo en la capa** (no en circuito):

| Invariante | Dónde | Riesgo si falla |
|---|---|---|
| No se reaplica una operación | Comparación de `root_old` | Duplicación de dinero |
| El límite es el del sistema | `apply` | El regulado elegiría su límite |
| El estado no está corrupto | `load` | Pruebas válidas sobre un ledger falso |

---

## 4. FALLO GRAVE SIN RESOLVER: el pagador ve el saldo del receptor

**Encontrado durante esta sesión, al empezar un refactor por otro motivo.**

### Qué ocurre

`TransferMaterials` —lo que la capa entrega al cliente para que genere su
prueba— incluye `receiver: AccountView`, que contiene:

- El **saldo exacto** del receptor
- Su **identidad pública**
- Su **nonce**

**Pagar un euro a alguien revela cuánto tiene.**

### Por qué es más grave que la visibilidad del operador

| | Operador | **Contraparte** |
|---|---|---|
| Quién | Una entidad | **Cualquiera que reciba un pago tuyo** |
| ¿Declarado? | Sí, en cabecera de todos los documentos | **No lo estaba** |
| Mitigable con confianza institucional | Sí | **No** |

### Por qué existe

Es inherente al **modelo de cuentas** con un solo probador: la liquidación
actualiza las dos hojas, así que quien construye la prueba necesita el
saldo del receptor para calcular su hoja nueva.

Zcash no lo tiene porque **no actualiza el saldo del receptor**: crea una
nota nueva para él.

### Cómo se dejó pasar

El comentario del código lo justificaba:

> *"Son caminos de Merkle y datos de cuenta: información de estado, no
> secretos."*

Esa frase **da por buena la fuga en vez de examinarla**. Y no hay ningún
test que compruebe qué aprende el pagador — todos los tests de privacidad
miran qué ve un tercero que solo tiene la prueba.

**Un test que hubiera preguntado *"¿qué sabe el pagador después de
pagar?"* lo habría delatado desde el principio.**

### Salidas

| Vía | Coste | Compatible con los principios |
|---|---|---|
| Protocolo entre dos partes: cada uno prueba su lado | Interacción, disponibilidad del receptor | Sí |
| Modelo de notas tipo UTXO | Rediseño del sistema entero | Sí |
| Que la capa construya la prueba | Barato | **No: viola P5** |

### La vía elegida, con el diseño demostrado

**Transferencia en dos fases.** El pagador **no toca la hoja del
receptor**: crea un compromiso pendiente atado a su identidad, y el
receptor lo reclama.

```text
FASE 1 (pagador)             FASE 2 (receptor)
· debita su cuenta           · demuestra que el pendiente es SUYO
· crea el pendiente          · acredita su cuenta
  P = H(H(id_r, s), importe) · lo anula
```

| Parte | Necesita | NO necesita |
|---|---|---|
| Pagador | La identidad pública del receptor, como dirección | **Su saldo. Ni su nonce.** |
| Receptor | Su propio estado y el aviso | Nada del pagador |
| Un tercero | — | Ve un compromiso opaco |

**Demostrado en `pending.rs` con 8 tests**, incluidos los cuatro ataques:
reclamar el pendiente ajeno, reclamarlo el pagador, reclamarlo dos veces, e
inflar el importe.

Y la propiedad va **en el tipo**, no solo en un test: la firma
`create(receiver_id, salt, amount)` **no admite un saldo**. No hay dónde
meterlo.

⚠️ **El residuo**: el pagador elige el aleatorio, así que **reconoce cuándo
se reclama el pendiente**. Sabe *cuándo* cobra el receptor, no cuánto
tiene. Probado en `the_payer_can_still_tell_when_it_is_claimed`.

⚠️ **El coste**: el pago pasa a dos pasos y el dinero **queda pendiente
hasta que el receptor actúe**. Un despliegue real necesitaría reclamación
automática por su proveedor — con lo que el proveedor vuelve a ver el
saldo, aunque repartido y no concentrado en el operador.

### Estado: **RESUELTO**

| Pieza | Estado |
|---|---|
| Diseño (`pending.rs`) | ✅ 8 tests |
| `circuit_send` — debita y crea el pendiente | ✅ 12 tests |
| `circuit_claim` — demuestra que es suyo y cobra | ✅ 13 tests |
| Capa: `send`, `claim`, persistencia | ✅ 5 tests de integración |

**La garantía está en el tipo, no en un test:**

```rust
pub fn send(&self, spend_key, sender_index, receiver_id, salt, amount)
```

**No hay parámetro donde pudiera entrar el saldo del receptor.** Un
descuido futuro no puede reintroducir la fuga sin cambiar la API — que es
la lección de los dos hallazgos que se reintrodujeron por escribir código
nuevo sin consultar lo aprendido.

### Los ataques cerrados

| | |
|---|---|
| Quien intercepte el aviso | **No cobra**: le falta la clave |
| Reclamar dos veces | El pendiente queda consumido, impuesto en circuito |
| Enviar estando congelado | Rechazado |
| Reiniciar el nodo | Los pendientes sobreviven |

### ⚠️ Lo que sigue abierto

**La vía antigua sigue existiendo.** `transfer()` conserva la fuga, con su
advertencia en el código. Retirarla exige migrar lo que la use, y **no está
hecho**.

**El aviso viaja fuera del sistema.** El receptor necesita el aleatorio y
el importe; la capa no los transporta. **Perder el aviso es no poder
reclamar** aunque el dinero esté ahí.

**Y el residuo de vinculabilidad**: el pagador eligió el aleatorio, así que
reconoce el compromiso y **ve cuándo desaparece del árbol**. Sabe *cuándo*
cobra el receptor, no cuánto tiene.

**Y una discrepancia detectada al integrar**: `circuit_settlement`
incrementa el nonce; `circuit_burn`, `circuit_send` y `circuit_claim` no.
La protección contra reenvío de estos viene del encadenamiento de raíces.
**Tres circuitos con dos comportamientos, y nada lo decía.**

---|---|
| Diseño demostrado (`pending.rs`) | ✅ 8 tests |
| **`circuit_send`** — el pagador debita y crea el pendiente | ✅ **12 tests** |
| **`circuit_claim`** — el receptor demuestra que es suyo y cobra | ✅ **13 tests** |
| La capa: dos fases, pendientes, reclamación | ⬜ 3-5 rondas |

**La propiedad va en el tipo, no solo en un test.** La firma de
`build_trace` recibe `receiver_id` y `salt`: **no hay parámetro donde
pudiera entrar un saldo del receptor**.

Los dos ataques que sostienen el esquema, cerrados con test:

- `nobody_else_can_claim_a_pending_transfer`: quien intercepte el aviso
  —aleatorio e importe— **no puede cobrarlo sin la clave**.
- La raíz de pendientes queda vacía tras reclamar: **no se cobra dos
  veces**.

### La vía elegida, analizada

**Transferencias por notas.** El pagador crea una nota atada a la identidad
del receptor; el receptor la reclama. El pagador **nunca ve su saldo**.

Las piezas existen en el proyecto derivado —`circuit_issue` y
`circuit_redeem`, escritas para el modo sin conexión— y resuelven esto sin
cambios.

⚠️ **Residuo**: el pagador elige el aleatorio de la nota, así que
**reconoce cuándo se reclama**. Sabe cuándo cobra el receptor, no cuánto
tiene. Zcash lo cierra cifrando la nota; aquí no está resuelto.

⚠️ **Coste**: la transferencia pasa a dos pasos y **el receptor tiene que
actuar** para cobrar.

**Estado: diseño analizado, sin implementar** (15-25 rondas estimadas).

### Qué debe hacer un auditor con esto

Comprobar si hay más fugas del mismo tipo: **datos que se entregan al
cliente por necesidad técnica y que nadie examinó**. La pregunta correcta
no es *"¿qué ve un observador?"* sino **"¿qué aprende cada participante?"**.

---

## 5. Qué aprende cada participante

**Repaso sistemático aplicando la pregunta de §4**: no *"¿qué ve un
observador?"* sino **"¿qué aprende cada participante?"**.

Encontró la fuga de §4 y no encontró otra igual de grave. Pero sí tres
cosas que estaban implícitas y **ninguna declarada**.

### 11.1 Matriz por operación

| Operación | El operador | La contraparte | Un tercero |
|---|---|---|---|
| **Transferencia** | Todo | **El saldo del receptor** ⚠️ §4 | Nada |
| Emisión | Todo | Custodios: **el importe** | **El importe** |
| Destrucción | Todo | — | **El importe** |
| Auditoría | Lo revelado | Supervisor: **la identidad de la cuenta** | Nada |
| Recuperación | Todo | Custodios: la identidad nueva | Nada |
| Congelación | Todo | Custodios: **qué cuenta** | Nada |
| Gasto sin conexión | — | Comercio: el importe | Nada |

### 11.2 Lo implícito, ahora declarado

**El importe de emisión y destrucción es público.** Aparece en las entradas
públicas de ambos circuitos.

Es **deliberado**: el suministro total tiene que ser verificable, y sin
importes públicos no lo sería. Pero significa que **cualquiera ve cuánto
dinero se crea y se destruye, y cuándo**. No ve a quién.

**La revelación selectiva identifica la cuenta.** `AuditPublicInputs`
incluye `public_id`.

Es necesario —el supervisor debe saber de quién es el saldo que
comprueba— pero tiene una consecuencia: **dos revelaciones del mismo
titular son vinculables entre sí**. No se puede demostrar
anónimamente *"alguien cumple el límite"*.

**El registro de transiciones revela la secuencia de tipos de operación.**
Cada entrada lleva su tipo: emisión, transferencia, destrucción,
congelación.

Un observador del registro ve **el patrón de actividad del sistema**
—cuántas transferencias, cuándo hubo congelaciones— sin ver a quién ni
cuánto. Es metadato, y en un sistema con poca actividad podría ser
significativo.

### 11.3 Lo que el repaso confirmó que SÍ está cerrado

| Propiedad | Cómo |
|---|---|
| La emisión no revela la cuenta acreditada | Solo raíces, no identidades |
| La destrucción no revela la cuenta | Ídem |
| El nullifier no se deriva de la identidad pública | Test: `nullifier_is_not_derivable_from_public_id` |
| Una transferencia no revela identidades a terceros | Solo raíces y nullifier |

### 11.4 La pregunta que queda para un auditor

**¿Hay más datos que se entregan por necesidad técnica y que nadie
examinó?**

La fuga de §4 existía porque un comentario la justificaba —*"información de
estado, no secretos"*— en vez de examinarla. **Ese patrón puede repetirse
en sitios que este repaso no ha mirado**: los materiales de auditoría, o
cualquier cosa que se añada después.

### 5.5 Un sitio ya mirado: las instantáneas

**Se encontró una incoherencia y está cerrada.**

La base de datos se cifraba con XChaCha20-Poly1305 —dieciséis llamadas a
`seal`— y **la instantánea iba en claro**. Dos niveles de protección
distintos para el mismo dato, y la instantánea es justo **el artefacto que
se copia fuera del nodo**: a una cinta, a otro servidor, a un disco que
alguien se lleva.

Ahora va cifrada con la misma clave, con un byte de marca al principio para
que la importación sepa qué tiene delante sin adivinar. **5 tests**,
incluido el que busca el saldo byte a byte en el fichero y su validador.

⚠️ **Sin clave sigue yendo en claro**, y entonces quien tenga el fichero ve
todos los saldos. Es coherente —sin clave tampoco se cifra el ledger— pero
conviene saberlo.

**Lo que enseña**: la incoherencia no estaba en ninguna función, estaba
**entre dos funciones**. Aparece al preguntar *"¿qué protección tiene cada
artefacto que contiene el mismo dato?"*, no al revisar el código de una en
una.

---

## 6. Un fallo sistemático encontrado y corregido: mutar antes de comprobar

**Las cinco operaciones que modifican árboles mutaban el estado antes de
comprobar que la raíz resultante coincidía con la que la prueba acredita.**

```rust
self.accounts.set_leaf(...);          // ← muta
self.total_supply = ...;              // ← muta

if self.accounts.root() != pi.root_new {
    return Err(StaleState);           // ← error, PERO YA MUTÓ
}
```

### Qué implicaba

| | |
|---|---|
| ¿Se persistía? | **No**: `commit()` no llegaba a ejecutarse |
| ¿Quedaba en memoria? | **Sí, hasta reiniciar** |
| Consecuencia | El nodo operaba con **un estado que no correspondía a su disco** |

En `transfer` era peor: dos hojas de cuenta y un nullifier, así que un
fallo dejaba **tres cosas cambiadas**.

### El caso concreto que lo destapó

Un recibo de emisión para una cuenta, **aplicado sobre otra**. La prueba
verifica —no dice qué cuenta— y la raíz vieja coincide, así que la
comprobación solo falla al final. Para entonces la otra cuenta ya tenía el
importe sumado en memoria.

### Cómo se encontró

Un test escrito para otra cosa: `applying_a_receipt_to_the_wrong_account_is_rejected`,
al preguntar **"¿qué pasa si se reenvía un recibo?"**. No buscaba esto.

### Lo corregido

Las cinco operaciones aplican sobre **una copia** y solo confirman si la
raíz cuadra. Es el patrón que `commitment.rs` ya usaba correctamente —
escrito después, pensando el diseño desde cero— mientras las cinco
existentes lo repetían mal sin que nadie lo mirara.

### Lo que un auditor debería extraer

**El fallo no estaba en ninguna operación: estaba en las cinco.** Un
patrón incorrecto copiado entre funciones no se ve revisándolas de una en
una, porque cada una parece coherente consigo misma.

Aparece al preguntar **"¿qué pasa si esto falla a mitad?"** — una pregunta
que se hace al conjunto, no a la función.

---

## 7. Qué bloquea de verdad una congelación

**Aplicación de P4 —medida: el sistema define qué resuelve y qué rechaza—
a la pregunta de qué estados alcanzan las combinaciones de operaciones.**

### 11.1 La matriz, probada

| Con la cuenta congelada | ¿Se permite? | Dónde se impone |
|---|---|---|
| Transferir | **No** | Circuito (42 restricciones) + capa |
| **Destruir** | **No** | **Circuito (13 restricciones) + capa** |
| Recibir | **Sí**, deliberado | — |
| Ser auditada | Sí | — |
| Recibir una emisión | Sí | — |
| Ser recuperada | Sí, **y la congelación sobrevive** | Probado |

### 11.2 El hueco que había

**Congelar bloqueaba transferir y no bloqueaba destruir.** Un titular bajo
investigación podía **vaciar su cuenta a cero**: no se llevaba el dinero
—se destruía— pero el saldo investigado desaparecía.

El circuito de liquidación miraba el árbol de congelados; el de destrucción
**no lo miraba en absoluto**, y la capa tampoco.

### 11.3 La decisión, y su razonamiento

**Congelar existe para que una cuenta bajo investigación no mueva fondos.
Destruirlos los mueve: los saca del sistema. Que sea público e irreversible
no los devuelve.**

Implementado: `circuit_burn` gana una fase de **no-pertenencia al árbol de
congelados** —24 niveles, filas 280..471, que estaban libres— con 13
restricciones nuevas y **3 tests**, incluido el validador que comprueba que
una cuenta libre sí puede.

### 11.4 Por qué se permite recibir

Impedir que una cuenta congelada **reciba** dejaría fondos en el limbo y
rompería pagos legítimos hacia alguien bajo investigación. Es una decisión
deliberada, no un olvido.

### 11.5 Por qué la recuperación no la levanta

El árbol de congelados se indexa por **posición de cuenta**, no por
identidad. Si se indexara por identidad, bastaría con decir que se perdió
la clave para escapar de una investigación.

Era una consecuencia del diseño, no una decisión probada. **Ahora hay un
test.**

### 7.6 ⚠️ Lo que esta pregunta NO ha cubierto

Se han probado las combinaciones con **la congelación** porque es el
privilegio con más superficie. Quedan sin explorar sistemáticamente:

- Combinaciones con el **límite regulatorio** y el tope de emisión.
- Combinaciones que atraviesan **reinicios** del nodo.
- Combinaciones de **tres o más** operaciones.

**No hay motivo para pensar que estén limpias**: esta pregunta encontró un
hueco a la primera.

---

## 8. El método: nueve preguntas transversales

**Todos los hallazgos de esta ronda de revisión salieron de preguntas que
se hacen al conjunto, no a una función.** Ninguna se responde leyendo un
fichero, y por eso ninguna había salido antes pese a que todo el código
compilaba y pasaba sus tests.

| Pregunta | Encontró |
|---|---|
| ¿Qué aprende cada participante? | **El pagador ve el saldo del receptor** |
| ¿Frente a quién es cierta esta afirmación de privacidad? | **El banco enlaza los pagos sin conexión** |
| ¿Qué protección tiene cada artefacto con el mismo dato? | Las instantáneas iban sin cifrar |
| ¿Qué se comprueba antes de autorizar? | La congelación se filtraba a no titulares |
| ¿Y si se reenvía un recibo? | Faltaban dos tests, uno de ellos crearía dinero |
| ¿Y si una operación falla a mitad? | **Cinco operaciones dejaban el estado corrupto** |
| ¿Qué dice la documentación que el código no hace? | Cinco cifras y una limitación ya inexistente |
| ¿Qué estados alcanzan las combinaciones? | **Una cuenta congelada podía destruir su dinero** |
| ¿Qué puede el operador que no esté declarado? | **Puede desviar un pago si no compruebas el destino** |

### 11.1 Lo que tienen en común

**Ninguna se responde revisando una función.** Cada una compara cosas que
son coherentes por separado:

- Dos artefactos con el mismo dato y distinta protección.
- Cinco operaciones que repiten un patrón incorrecto, cada una consistente
  consigo misma.
- Una afirmación de privacidad cierta contra un adversario y falsa contra
  otro.

### 8.2 ⚠️ Y la conclusión incómoda

**Nueve preguntas, nueve hallazgos.** En código que compilaba, pasaba
todos sus tests, y llevaba meses escrito.

No hay motivo para pensar que las preguntas se hayan acabado. Las que
quedan sin hacer son las que no se me han ocurrido — y esas son
precisamente las que encontraría alguien de fuera.

**Es el argumento más fuerte a favor de una auditoría externa que contiene
este documento**, y no es retórico: es el registro de lo que pasó al
preguntar en serio.

---

## 9. Un fallo de método que costó ocho rondas

**Todos los circuitos tienen un test de puntos de referencia que compara la
traza con su cálculo nativo. Ninguno comparaba TODAS las entradas
públicas.**

### Qué pasó

Al construir `circuit_send`, el escenario de prueba heredó de
`circuit_burn` la línea `supply_new = supply_old − amount`. Un envío **no
cambia el suministro**, así que la traza tenía `supply_new = supply_old`.

Entradas públicas distintas entre probador y verificador → **transcripciones
de Fiat-Shamir distintas** → la prueba se genera y **no verifica**.

### Por qué costó tanto

El error de winterfell es `InconsistentOodConstraintEvaluations`, que
**apunta a las restricciones**. Se descartaron, en este orden: la traza, las
violaciones de restricción, los índices de columnas periódicas, el recuento
de aserciones y los grados declarados.

Todas eran hipótesis razonables. Ninguna era la causa.

⚠️ **Y dos conclusiones se anunciaron y hubo que retirarlas**, incluida una
—*"doblar la traza rompe `circuit_burn`"*— que resultó ser un artefacto de
la propia bisección: al revertir el bucle no se revirtió el indicador de
hash.

### La corrección

```rust
let derivadas = Prover::new(opts).get_pub_inputs(&trace);
assert_eq!(derivadas.to_elements(), declaradas.to_elements());
```

**Comparar la estructura entera**, no los campos que parecen importantes.

### ⚠️ Aplica a los otros nueve circuitos

Todos comparan raíces seleccionadas a mano. **Ninguno compara
`to_elements()` completo.** El mismo fallo puede estar latente en cualquiera
de ellos y solo aparecería al modificarlo.

Corregirlo son 1-2 rondas por circuito, y **no está hecho**.

---

## 10. Columnas declaradas que nunca se rellenan

**Encontrado al construir `circuit_mint_pending`**, preguntando *"¿cada
columna que declaro se usa de verdad?"*.

Siete columnas nuevas —suministro, tope, importe, identidad del receptor,
aleatorio, bit de dirección— estaban **declaradas, con sus restricciones
escritas, y la traza nunca les ponía valor**.

### Por qué ningún test lo detecta

Todas valían cero, así que sus restricciones se cumplían trivialmente:
`0 − 0 = 0`.

**Los tests negativos pasaban igual**, porque fallaban por otras
restricciones. Y el testigo honesto no revela nada, porque en su caso las
columnas *deberían* tener valor y él las rellenaría... si el código las
rellenara.

Es el mismo modo de fallo que las **restricciones idénticamente cero**
documentadas en §7 del primer proyecto. **Tercera vez que aparece.**

### Y destapó un segundo fallo

El **tope de emisión** se transporta y se declara públicamente, pero
**nunca se comprueba con un rango**. La cabecera del módulo afirmaba que sí
y hubo que corregirla.

⚠️ **Sigue sin corregirse**: cerrarlo exige un segmento de rango más.

### Lo que un auditor debería hacer

**Comprobar, para cada columna de cada circuito, que la traza le asigna
valor.** Es mecánico y no hay nada automático que lo haga.

Este proyecto tiene **once circuitos** y esta comprobación solo se ha hecho
en uno.

---

## 11. Donde el autor tiene MENOS confianza

Esta es la sección más útil del documento.

### 11.1 `open_account` no exige autorización — **mitigado a medias**

Cualquiera con acceso a la capa puede crear cuentas. No crea dinero
(nacen a cero), pero llenaba el árbol y el mapa de registros hasta agotar
la memoria.

**Mitigación aplicada**: tope de cuentas inmutable (`max_accounts`).

**Lo que NO resuelve**: un atacante puede agotar el cupo y dejar sin sitio
a usuarios legítimos. La solución correcta —exigir autorización de
custodio para abrir— requiere un circuito nuevo, porque **abrir hoy no
genera ninguna prueba**.

Un auditor debería valorar si el tope es suficiente para el caso de uso
previsto.

### 11.2 La congelación no tiene justificación ni caducidad

**Implementada** con imposición en circuito: la prueba de liquidación
acredita que el emisor no está en el árbol de congelados.

**Lo que queda abierto:**

- El circuito demuestra que dos custodios la autorizaron, **no que
  tuvieran razón**. No hay orden judicial ni motivo registrado.
- **No hay caducidad**: una congelación dura hasta que alguien la levante.
- Una cuenta congelada **sigue pudiendo recibir**. Es deliberado —lo
  contrario dejaría fondos en el limbo— pero merece que un auditor valore
  si encaja con el caso de uso.

### 11.3 Los grados de restricción

**Cinco veces** durante el desarrollo winterfell rechazó un grado mal
declarado. Cada vez se corrigió. La exactitud que exige winterfell hace
improbable que un grado incorrecto pase inadvertido, **pero la
concentración de errores en este punto sugiere revisarlo con atención**.

Especial cuidado con:
- Restricciones que multiplican **dos columnas periódicas** (`C_ACC`).
- Columnas periódicas cuyo periodo real difiere del declarado
  (`circuit_mint`: 8 segmentos × 64 filas llenan la traza y la vuelven
  periódica de periodo 64).

### 11.4 El patrón lockstep

`C_SIBLING` impone que los dos carriles usen el mismo hermano. El
argumento es que eso basta para atar ambas subidas a la misma posición
del árbol.

Está verificado con un test discriminante, **pero el argumento general no
ha sido revisado por nadie más**. Es el hallazgo más original del
proyecto y merece escrutinio.

### 11.5 Los tests negativos

**Tres veces** un test negativo resultó no discriminar: fallaba por una
restricción distinta de la que pretendía probar. Se corrigieron
construyendo testigos internamente coherentes.

**Puede quedar alguno más.** Un auditor debería comprobar, para cada test
negativo, que el testigo corrupto es válido en todo lo demás.

### 11.6 El bloqueo de directorio de `sled` tras cerrar — **hallazgo nuevo**

`sled` mantiene un bloqueo del directorio que puede tardar en liberarse
tras cerrar la base de datos. **Un nodo que se reinicie inmediatamente
tras apagarse puede fallar al abrir** con un error de E/S.

Se descubrió porque un test que reabre un ledger fallaba de forma
intermitente —2 de 3 ejecuciones bajo carga paralela, pero nunca en
aislamiento—.

**No está mitigado en la capa**: `SovereignLayer::open` devuelve el error
sin reintentar. Un despliegue real debería reintentar con espera, como
hace el ayudante `open_retry` de los tests.

Un auditor debería valorar si esto afecta a los procedimientos de
recuperación tras caída.

### 11.7 El techo de 63 bits

Las comprobaciones de rango fuerzan el bit más significativo a cero, así
que **ningún valor puede superar 2^63 − 1**.

La capa **no valida** que `max_supply` esté por debajo de ese techo. Con
un tope mayor, las emisiones fallarían con un error confuso en vez de
rechazarse al configurar.

No es una fuga de solidez —los valores fuera de rango se rechazan— pero
sí un fallo de usabilidad que puede confundir un diagnóstico.

### 11.8 El formato de instantánea se queda atrás al añadir estado

**Dos veces** en pocas rondas: al añadir las cuentas congeladas y al
añadir el registro de transiciones, la instantánea dejó de incluir algo
que debía conservar. Las dos veces se detectó **después** de escribir la
funcionalidad.

Es una debilidad de **proceso**: el formato enumera campos a mano y nada
obliga a actualizarlo. Un test que compare los campos de `SovereignLayer`
con los que serializa la instantánea lo habría cazado las dos veces, y no
existe.

Un auditor debería comprobar que la versión actual del formato cubre todo
el estado, y valorar exigir ese test.

### 11.9 Colisiones en el árbol de nullifiers

La posición sale de los bits bajos del nullifier. Dos nullifiers pueden
colisionar, y el segundo **no podría gastarse**.

El autor lo clasifica como **denegación de servicio, no ruptura de
solidez**. Ese razonamiento merece verificación independiente: si fuera
incorrecto, sería grave.

---

## 12. Por dónde empezaría el autor si tuviera que romperlo

En este orden:

1. **`C_NULL_EMPTY` y las raíces del árbol de nullifiers.** Es la
   protección contra doble gasto y la más reciente.
2. **La interacción entre `nonce`, nullifier y recuperación.** Tras
   recuperar, el nonce incrementa. ¿Puede eso colisionar con un nullifier
   ya gastado o invalidar uno legítimo?
3. **`C_ACC` en los circuitos de umbral.** Si el acumulador no atara el
   índice al camino, el orden estricto no valdría nada y un custodio
   podría firmar dos veces.
4. **El orden de las comprobaciones en `apply`.** Si alguna se hiciera
   después de mutar el estado, un fallo dejaría el ledger inconsistente.
5. **La serialización en `store.rs`.** Un dato mal formado que se
   interpretara en vez de rechazarse produciría valores plausibles pero
   falsos.

---

## 13. Limitaciones ya documentadas

No hacen falta descubrirlas; están en `README.md`:

- El operador del nodo es un intermediario de confianza.
- No hay red, consenso, réplicas ni cifrado en reposo.
- La generación de la prueba puede hacerse en el cliente (`client`), pero
  delegarla a un **tercero** exigiría verificar una firma en circuito.
- La resolución IBAN → cuenta está fuera de la prueba.
- El conjunto de gobernanza es inmutable.
- Las cifras de rendimiento son de una sola ejecución.

---

## 14. Cómo reproducir

### ⚠️ Hay un test ignorado, y conviene saber por qué

```
cargo test -p stark-experiment --release
→ 135 passed; 1 ignored
```

El ignorado está en `range_check.rs`: comprueba el **caso cero**, y en
compilación de depuración winterfell rechaza sus grados porque la traza
degenera —todos los bits a cero—. Está marcado `#[ignore]` para que la
suite pase limpia en debug.

**Ejecutarlo exige release:**

```bash
cargo test -p stark-experiment --release -- --ignored
```

⚠️ Un test ignorado **es un test que normalmente no se ejecuta**. Que esté
justificado no lo hace inofensivo: si su comportamiento cambiara, nadie se
enteraría hasta que alguien lo ejecutara a propósito.

Un auditor debería ejecutarlo y valorar si el caso cero está bien cubierto.



```bash
cargo test -p zk-ssl --release              # la capa, 65 tests
cargo test -p stark-experiment --release    # los seis circuitos
cargo test -p zk-ssl --release metrics -- --nocapture
```

**Los tests de circuito conviene ejecutarlos también en debug**: winterfell
valida las restricciones al generar solo en ese modo, y da el índice y la
fila exactos del fallo.

---

## 15. Qué NO demuestra este documento

Que el sistema sea seguro. Demuestra que **el autor ha buscado sus
propios fallos de forma sistemática y ha encontrado algunos**, incluidos
dos al escribir estas páginas.

Es exactamente por eso que hace falta que lo mire alguien más.
