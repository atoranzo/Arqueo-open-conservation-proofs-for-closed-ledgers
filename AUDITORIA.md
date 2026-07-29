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

### 16.1 Matriz por operación

| Operación | El operador | La contraparte | Un tercero |
|---|---|---|---|
| **Transferencia** | Todo | **El saldo del receptor** ⚠️ §4 | Nada |
| Emisión | Todo | Custodios: **el importe** | **El importe** |
| Destrucción | Todo | — | **El importe** |
| Auditoría | Lo revelado | Supervisor: **la identidad de la cuenta** | Nada |
| Recuperación | Todo | Custodios: la identidad nueva | Nada |
| Congelación | Todo | Custodios: **qué cuenta** | Nada |
| Gasto sin conexión | — | Comercio: el importe | Nada |

### 16.2 Lo implícito, ahora declarado

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

### 16.3 Lo que el repaso confirmó que SÍ está cerrado

| Propiedad | Cómo |
|---|---|
| La emisión no revela la cuenta acreditada | Solo raíces, no identidades |
| La destrucción no revela la cuenta | Ídem |
| El nullifier no se deriva de la identidad pública | Test: `nullifier_is_not_derivable_from_public_id` |
| Una transferencia no revela identidades a terceros | Solo raíces y nullifier |

### 16.4 La pregunta que queda para un auditor

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

### 16.1 La matriz, probada

| Con la cuenta congelada | ¿Se permite? | Dónde se impone |
|---|---|---|
| Transferir | **No** | Circuito (42 restricciones) + capa |
| **Destruir** | **No** | **Circuito (13 restricciones) + capa** |
| Recibir | **Sí**, deliberado | — |
| Ser auditada | Sí | — |
| Recibir una emisión | Sí | — |
| Ser recuperada | Sí, **y la congelación sobrevive** | Probado |

### 16.2 El hueco que había

**Congelar bloqueaba transferir y no bloqueaba destruir.** Un titular bajo
investigación podía **vaciar su cuenta a cero**: no se llevaba el dinero
—se destruía— pero el saldo investigado desaparecía.

El circuito de liquidación miraba el árbol de congelados; el de destrucción
**no lo miraba en absoluto**, y la capa tampoco.

### 16.3 La decisión, y su razonamiento

**Congelar existe para que una cuenta bajo investigación no mueva fondos.
Destruirlos los mueve: los saca del sistema. Que sea público e irreversible
no los devuelve.**

Implementado: `circuit_burn` gana una fase de **no-pertenencia al árbol de
congelados** —24 niveles, filas 280..471, que estaban libres— con 13
restricciones nuevas y **3 tests**, incluido el validador que comprueba que
una cuenta libre sí puede.

### 16.4 Por qué se permite recibir

Impedir que una cuenta congelada **reciba** dejaría fondos en el limbo y
rompería pagos legítimos hacia alguien bajo investigación. Es una decisión
deliberada, no un olvido.

### 16.5 Por qué la recuperación no la levanta

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

### 16.1 Lo que tienen en común

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

✅ **Corregido.** Se añadió un segmento de 64 filas (filas 320-383, que
estaban vacías) con dos columnas nuevas y seis restricciones. Dos tests lo
fijan: `minting_beyond_the_cap_is_rejected` y
`minting_exactly_up_to_the_cap_is_allowed`.

⚠️ **El par es necesario.** La primera versión solo tenía el negativo, y
**pasaba por la razón equivocada**: el circuito rechazaba *cualquier*
emisión porque el test reutilizaba unas entradas públicas de importe fijo.
Un test negativo solo dice algo cuando su positivo lo acompaña.

### Lo que un auditor debería hacer

**Hecho, y automatizado**: `tools/check_columns.py`.

```
python3 tools/check_columns.py crates/stark-experiment/src
→ 11 circuitos: todas las columnas declaradas se rellenan.
```

⚠️ **La comprobación se equivocó dos veces antes de funcionar.**

La primera versión solo contaba lecturas de la forma `current[COL]` y **no
detectaba** el fallo que motivó escribirla. La segunda no reconocía el
patrón `state[COL] = …` de `trace.fill` y daba **cinco falsos positivos**.

Ambos errores salieron al **validar la comprobación contra un caso
conocido**: quitar a propósito el relleno de una columna y confirmar que
salta, y comprobar que un circuito correcto no dispara.

> **Una verificación rota es peor que ninguna**: no solo no detecta, sino
> que dirige la atención al sitio equivocado.

⚠️ **Lo que la herramienta NO comprueba**: que el valor sea el correcto, que
se rellene en todas las filas donde hace falta, ni columnas con un patrón
de relleno distinto a los tres reconocidos.

### El mismo fallo en las RESTRICCIONES: se intentó y no funciona

Una restricción declarada y **nunca asignada** queda a cero e impone lo
mismo que una columna vacía: nada. Se intentó la comprobación equivalente y
**no puede hacerse con análisis de texto**.

El motivo son dos casos indistinguibles mirando el código:

| Constante | Forma | Realidad |
|---|---|---|
| `C_HASH_B` | Solo aparece definiendo `C_CAP_A` | Su rango **sí** se escribe: `result[C_HASH_A + lane*12 + i]` |
| `C_PBIT_BOOL` sin asignar | Solo aparece definiendo `NUM_CONSTRAINTS` | Su rango **no** se escribe: es un fallo |

Distinguirlos exige resolver qué índices cubre cada escritura con índice
calculado — análisis de rangos, no expresiones regulares.

**La técnica que sí funcionaría**: *perturbar el testigo y ver qué
restricciones reaccionan*. Una que no reaccione a ninguna perturbación no
impone nada.

Es **prueba por mutación**, exige generar perturbaciones significativas por
circuito, y **no está hecho**.

⚠️ Se prefirió **no publicar una herramienta rota**: una versión que
clasificara mal estos casos daría confianza infundada, que es peor que no
tener comprobación.

---

## 11. Qué módulos importan, y cuáles no

**De 24 módulos en `stark-experiment`, la capa de producción usa 13.** Los
otros 11 son del estudio comparativo entre paradigmas.

Nadie lo había separado, y eso hace que las auditorías se apliquen sin
priorizar. **Un auditor con tiempo limitado debe saber cuáles importan.**

### Los 13 de producción

`circuit_audit`, `circuit_burn`, `circuit_claim`, `circuit_freeze`,
`circuit_governance`, `circuit_mint`, `circuit_mint_pending`,
`circuit_recovery`, `circuit_send`, `circuit_settlement`,
`circuit_threshold`, `nullifier_tree`, `merkle`.

### Los 11 del estudio

`compliance_circuit`, `compliance_real_proof`, `double_entry`,
`dual_climb`, `iso_bridge`, `nullifier`,
`persistent_nullifier_registry`, `range_check`, `rescue_hash`,
`settlement_prover_impl`, `solvency`.

Existen para la comparativa entre cinco paradigmas. **Un fallo en ellos no
afecta a la capa**, aunque sí a las conclusiones del estudio.

### Estado de los tests de puntos de referencia

**Los 12 comparan ahora todas sus entradas públicas** con lo que la traza
produce:

```rust
let derivadas = XProver::new(opts).get_pub_inputs(&trace);
assert_eq!(derivadas.to_elements(), declaradas.to_elements());
```

⚠️ **Tres no tenían ningún test de referencia** —`burn`, `audit`,
`governance`— pese a estar en producción. Se detectó al hacer este
inventario, no antes.

⚠️ Y los otros nueve **comparaban solo las raíces**, que es exactamente la
versión que dejó pasar el fallo de `circuit_send`.

### Qué protege y qué no

Protege contra **un campo de entradas públicas que el escenario declara
distinto de lo que la traza produce**. Ese fallo hace que probador y
verificador usen transcripciones de Fiat-Shamir distintas, y el error
resultante apunta a las restricciones.

**No protege** contra restricciones mal escritas, ni contra una traza que
satisface todo pero no significa lo que se cree.

⚠️ **Ningún circuito lo tenía cuando se escribió.** Se añadieron todos
después de que el fallo costara ocho rondas en uno de ellos.

### Por qué importa

Un test de puntos de referencia separa *"la traza está mal construida"* de
*"las restricciones están mal escritas"*. Sin él, un fallo de verificación
obliga a explorar las dos hipótesis a la vez — que es lo que costó **ocho
rondas** en `circuit_send`.

---

## 12. Prueba por mutación: buscar restricciones que no imponen nada

**Implementada** en `crates/stark-experiment/src/mutation.rs`, con un test
en `circuit_burn` (`no_constraint_is_vacuous`).

### La técnica

Si **ninguna perturbación** de una celda del testigo hace que una
restricción se vuelva no nula, esa restricción no impone nada.

```text
1. Construir una traza VÁLIDA.
2. Para cada celda: cambiarla.
3. Evaluar las dos transiciones afectadas.
4. Anotar qué restricciones se vuelven no nulas.
5. Las que nunca se disparan: vacías.
```

No genera ni una prueba: 19.968 celdas × 2 evaluaciones tardan 0,7 s.

### Validada en ambos sentidos

**Detecta**: anulando la comprobación de titularidad —dejándola
idénticamente cero, el modo de fallo que este documento describe— señala
**exactamente** los cuatro índices afectados, `[75, 76, 77, 78]`.

**No da falsos positivos**: con el circuito intacto, ninguna.

### ⚠️ Dos cosas que salieron al validarla

**El muestreo produce falsos positivos.** Probando una fila de cada cuatro
señaló ocho restricciones vacías que no lo eran: las activas en una sola
fila pueden quedar fuera de la muestra. **Hay que probar todas las filas o
tratar el resultado como sospecha, no como hallazgo.**

**Y una predicción del autor resultó falsa.** Se esperaba que, con la
titularidad anulada, el test `third_party_cannot_burn_someone_elses_money`
siguiera pasando — lo que habría demostrado que daba falsa confianza sobre
la propiedad que nombra.

**También falla.** Ese test sí cubre lo que dice cubrir.

### Qué añade entonces

Para restricciones **con** un test negativo que las cubra, nada: el test ya
las protege.

Añade valor donde **no hay** tal test. Este proyecto ha visto tres casos
—una restricción idénticamente cero, siete columnas nunca rellenadas, un
tope transportado sin comprobar— y ninguno lo detectaba nada.

### Cobertura y resultado

Aplicada a **los 12 circuitos de producción**. Los doce, **limpios**: toda
restricción declarada reacciona a alguna perturbación del testigo.

```
cargo test -p stark-experiment --release no_constraint_is_vacuous
→ 12 passed; 0 failed  (5,7 s)
```

`mint_pending` es el más significativo: es donde se declararon **siete
columnas que la traza nunca rellenaba**. Se corrigieron en su momento, y
ahora hay algo que comprueba que no quedó ninguna.

⚠️ **No ha encontrado ningún fallo.**

Eso es un resultado, aunque modesto: dice que estos seis circuitos no tienen
restricciones vacías, no que sean correctos. Y la herramienta está validada
—detecta el fallo cuando se introduce a propósito—, así que el resultado
limpio significa algo.

### Y sigue sin haber encontrado ningún fallo

Doce circuitos, ninguna restricción vacía. La herramienta está validada
—señala exactamente los índices cuando se introduce el fallo a propósito—
así que el resultado significa algo. Pero **el valor demostrado es
preventivo, no correctivo**: protege contra que alguien introduzca una
restricción vacía al modificar un circuito, no ha corregido ninguna.

⚠️ **Y la prueba por mutación no habría encontrado ese fallo**: el tope se
transportaba sin comprobarse, y una restricción que no existe no aparece
como vacía. La herramienta detecta restricciones declaradas que no imponen
nada, **no restricciones que faltan**.

⚠️ **No detecta** restricciones que solo reaccionan a cambios de varias
celdas a la vez, ni restricciones que se disparan pero imponen lo que no se
cree. **Un resultado limpio no significa que el circuito sea correcto.**

---

## 13. ⚠️ Capacidades declaradas frente a capacidades reales

**Es el hallazgo más grave de esta auditoría.** No de solidez —nadie roba
dinero— sino de **disponibilidad**, y con un mensaje de error que acusa al
usuario honesto de algo que no ha hecho.

### El mecanismo

La posición de un nullificador **se deriva del propio nullificador**:

```rust
pub fn nullifier_position(nullifier: &Digest) -> u64 {
    let v = nullifier[0].as_int();
    v & ((1u64 << TREE_DEPTH) - 1)   // TREE_DEPTH = 32
}
```

Y el circuito exige que esa posición esté **vacía** antes de insertar. Dos
nullificadores distintos que caigan en la misma posición son un conflicto,
y eso sigue la **paradoja del cumpleaños**:

| Nullificadores | Probabilidad de colisión |
|---|---|
| 10.000 | 1,2 % |
| **65.536** | **39 %** |
| 100.000 | **69 %** |
| 200.000 | **99 %** |

⚠️ **El árbol declara 4.294.967.296 posiciones. La capacidad práctica son
unos 65.000 pagos.**

### Qué le pasa al afectado

La capa responde:

```rust
if self.nullifiers.is_occupied(null_pos) {
    return Err(LayerError::NullifierAlreadySpent);
}
```

⚠️ **El pago legítimo queda bloqueado para siempre.** El nullificador es
determinista a partir del estado de la cuenta: no hay reintento posible ni
forma de elegir otra posición.

⚠️ **Y el error miente**: dice *"ya gastado"* cuando en realidad es una
colisión con el pago de otra persona. El usuario honesto ve una acusación
de doble gasto que es falsa.

En una moneda digital de banco central, **65.000 pagos son unos minutos**.

### La decisión, con los números

| Opción | Límite | Traza | Coste | Qué es |
|---|---|---|---|---|
| **A.** Dejar 32 | 65.536 pagos | 1024 | ×1 | Lo actual |
| **B.** Subir a 64 | ~4.300 millones | 2048 | ×2 | **Un aplazamiento** |
| **C.** Indexar por el nullifier completo | Sin colisiones **por construcción** | 4096 | ×4 | **La corrección** |

**Lo que distingue B de C**: B sigue truncando —hay probabilidad de
colisión, solo que baja—. C no trunca: dos nullifiers distintos ocupan
posiciones distintas **siempre**, y el modo de fallo pasa a ser una colisión
del hash, que es la suposición criptográfica que el sistema ya hace en todas
partes.

> **C mueve el problema al sitio donde debe estar. B lo aplaza.**

### ⚠️ Corrección de alcance: solo afecta a la vía ANTIGUA

Se dijo antes que el cambio *"toca cuatro circuitos"*. **Es falso.**

| Vía | ¿Usa nullificadores? | Límite del cumpleaños |
|---|---|---|
| `transfer()` — antigua | **Sí** | ~65.000 pagos |
| `send`/`claim` — nueva | **No** | **Ninguno** |
| `burn` | **No** | Ninguno |

`circuit_send` lo omite **por decisión documentada**: un envío cambia el
saldo, luego la hoja, luego la raíz, así que **un reenvío tendría la raíz
obsoleta y se rechazaría**. El nullificador no aporta nada ahí.

#### Lo que eso implica

**Retirar `transfer()` en favor de `send`/`claim` elimina el límite sin
tocar ningún circuito.** Y esa migración ya estaba decidida en §3.11 de
`VISION.md`: encaminar el puente ISO por `ACSP`/`ACSC`.

⚠️ **Dos problemas declarados por separado resultan ser el mismo.**

#### ⚠️ Pero vuelve en cuanto haya consenso

El propio `circuit_send` lo advierte:

> *Con varios validadores concurrentes esto cambiaría: el nullificador
> detecta un gasto repetido sin necesidad de ordenar.*

El encadenamiento de raíces exige un **orden total**. Un nodo único lo da;
un sistema distribuido, no. Así que el límite del cumpleaños **vuelve a
importar en cuanto se aborde la prioridad 5**, que sigue abierta.

### La decisión tomada: C es la correcta, y no está implementada

⚠️ **No se implementa ahora, y el motivo se declara**: cambiar `TREE_DEPTH`
toca `merkle.rs`, los cuatro circuitos que lo usan, las longitudes de traza
y **todas las constantes de fila** de cada uno. Es el tipo de cambio que en
esta misma auditoría ha producido, repetidamente, fallos silenciosos por
desplazamiento de índices.

**Hacerlo mal sería peor que no hacerlo**: un circuito que parece imponer
algo y no lo impone es exactamente el modo de fallo que §12 y §14
documentan.

⚠️ **Lo que sí cambia desde ahora**: este documento y los otros doce dejan
de describir el árbol de nullifiers como si tuviera 2³² de capacidad. B no
se ofrece como solución, porque no lo es.

### Lo que se corrigió al aplicar el hallazgo

**El error dejó de mentir.** La capa devolvía `NullifierAlreadySpent`, que
**acusaba al usuario honesto de un doble gasto que no había cometido**.
Ahora compara la hoja ocupada con el nullificador propio y distingue:

```rust
if self.nullifiers.leaf(null_pos) == nullifier {
    return Err(LayerError::NullifierAlreadySpent);
}
return Err(LayerError::NullifierPositionCollision { position: null_pos });
```

**Y el paper lo declara donde hace la afirmación**, no en una nota aparte.

### Y de paso apareció otro: el comodín del mapeo ISO

Al añadir el error nuevo, **el compilador no lo exigió**. Había un
`_ => ("TECH", ...)` que absorbía **9 de las 19 variantes**:

| Variante | Se reportaba como |
|---|---|
| ⚠️ **`AccountFrozen`** | **"problema técnico"** |
| `AccountLimitReached` | idem |
| `CustodianSetExhausted` | idem |
| `RecoveryToSameIdentity` | idem |
| ...y cinco más | idem |

⚠️ **`AccountFrozen` es el grave.** Un banco que recibe *TECH* reintenta;
uno que recibe *AC06* sabe que la cuenta está bloqueada. Decirle "problema
técnico" a un rechazo de negocio **es falso**, y en cumplimiento puede tener
consecuencias.

**El comodín se eliminó.** Ahora añadir un error nuevo **no compila** hasta
que alguien elija su código.

⚠️ **Pero los códigos concretos no están auditados.** `AC06` para cuenta
bloqueada es estándar; `MS03` y los `TECH` explícitos son decisiones
defendibles **que nadie ha verificado contra el catálogo ISO 20022**. Se
cambió un mapeo silencioso por uno explícito, no por uno correcto.

---

### El segundo: el árbol de pendientes se agota, y en menos de lo que parece

Sus posiciones **se asignan** con un contador secuencial, así que no tiene
el problema del cumpleaños. Pero el contador **solo sube: nunca reutiliza
las posiciones de los pendientes ya reclamados**.

⚠️ **El límite es de transferencias TOTALES desde el inicio, no
simultáneas.**

| Ritmo | Tiempo hasta agotar 2³² |
|---|---|
| 100 pagos/s | ~1,4 años |
| **1.000 pagos/s** | **~50 días** |
| 10.000 pagos/s | ~5 días |

Y al agotarse, `path_for` producía un camino que **no llega a la raíz**: la
prueba fallaba **sin decir por qué**.

**Corregido en parte**: `SparseTree::capacity()` declara el límite, y la
capa devuelve `PendingTreeExhausted` con su causa en vez de fallar de forma
inexplicable.

⚠️ **Pero el límite sigue existiendo.** Cerrarlo exige rotar el árbol o
reutilizar las posiciones liberadas al reclamar, y **no está hecho**.

---

### El patrón de esta sección

| Estructura | Capacidad declarada | Capacidad real |
|---|---|---|
| Nullificadores | 2³² posiciones | **~65.000 pagos** (cumpleaños) |
| Pendientes | 2³² posiciones | **2³² pagos totales**, no simultáneos |
| Cuentas | 2³² posiciones | 2³², correcto: se asignan y persisten |
| Congelados | 2²⁴ posiciones | 2²⁴, correcto |

**Lo que distingue a los dos primeros**: en uno la posición se *deriva*, en
el otro el contador *no recicla*. En ambos casos la capacidad anunciada por
la estructura **no es la del sistema**.

### Cómo se encontró

Aplicando el método de §14 a las profundidades de los árboles: **preguntar
qué capacidad tiene realmente cada estructura**, no cuánta declara.

`the_practical_capacity_is_the_birthday_bound_not_the_tree_size` fija la
aritmética para que se lea, no para que falle.

---

## 14. Invariantes frágiles: constantes acopladas sin declarar

`circuit_mint` comprueba el tope descomponiendo `tope − suministro_nuevo`
en bits. Si el suministro se pasara del tope, esa resta **envuelve** en el
campo de Goldilocks y da un valor cercano a `p ≈ 2^64`.

Que ese valor envuelto se rechace depende de que el segmento de rango dé
**63 bits y no 64**:

| | |
|---|---|
| Máximo representable con 63 bits | 9.223.372.036.854.775.807 |
| Valor envuelto de una resta negativa | ~18.446.744.069.000.000.000 |
| ¿Cabe? | **No** — por eso se rechaza |

⚠️ **Con 64 bits sí cabría.** El tope dejaría de imponerse y **ningún test
lo notaría**: los testigos honestos pasan igual y los adversariales fallan
antes por otras restricciones.

### De dónde sale el margen

De que `cont_s` marca `SEGMENT_LENGTH − 1 = 63` transiciones, no 64:

```rust
for p in 0..SEGMENT_LENGTH - 1 {
    cont_s[seg * SEGMENT_LENGTH + p] = one;
}
```

Partiendo de cero, 63 duplicaciones dan un valor de 63 bits.

⚠️ **Ese `- 1` parece un error de índice fuera por uno.** Quien lo
"corrigiera" dejaría el tope sin imponer, y el circuito seguiría pasando
todos sus tests.

### Ahora hay algo que lo detiene

`the_range_segment_is_63_bits_not_64` comprueba las dos cosas: que las
transiciones activas son `SEGMENT_LENGTH − 1`, y que con ese número de bits
el valor envuelto queda fuera del rango.

⚠️ **Se encontró preguntando por qué funciona antes de copiarlo**, no
revisando.

---

### El segundo: el tamaño del conjunto de custodios

El orden estricto entre custodios se impone descomponiendo
`idx_b − idx_a − 1` en los bits de un segmento. En los circuitos de
custodios ese segmento es de **8 filas = 7 bits**, hasta 127.

| `CUSTODIAN_DEPTH` | Custodios | Diferencia máxima | ¿Cabe? |
|---|---|---|---|
| **4** (actual) | 16 | 14 | Sí, con holgura |
| 7 | 128 | 126 | Sí, **justo** |
| **8** | 256 | 254 | **No** |

⚠️ **El techo está en 128 custodios**, y no lo decía nada.

**Y no sería un fallo de solidez, sino de disponibilidad**: con 256
custodios el circuito dejaría de admitir autorizaciones legítimas. Un
conjunto así funcionaría para índices cercanos y fallaría para los lejanos
— **un fallo intermitente que dependería de qué dos custodios firmaran**.

Con 256 custodios, en torno al 75 % de los pares seguiría funcionando. Es
el tipo de cosa que se descubre en producción.

`the_custodian_set_size_fits_the_range_segment` lo fija.

---

### El patrón

Los dos tienen la misma forma:

1. **Dos constantes acopladas** por una propiedad aritmética.
2. **El acoplamiento no está declarado** en ningún sitio.
3. **Ningún test lo cubre**: el sistema funciona con los valores actuales.
4. **Cambiar una de las dos** rompe algo que parece no relacionado.

⚠️ **Ninguna de las herramientas de esta auditoría los detecta.** No son
restricciones vacías ni columnas sin rellenar: son relaciones entre
constantes que solo se ven leyendo por qué funciona el mecanismo.

✅ **Y ya se replicó** en `circuit_mint_pending`, con una diferencia que el
comentario de `circuit_mint` avisaba: allí los 8 segmentos llenan la traza y
son periódicos de periodo 64; aquí es **un bloque único en 512 filas**, así
que el ciclo declarado es `TRACE_LENGTH`.

Leer por qué funciona antes de copiarlo evitó ese fallo.

---

## 15. La cifra de pruebas que este proyecto publica es incompleta

La documentación afirma **369 pruebas ejecutables** y da los dos comandos
que las ejecutan. Es preciso sobre **qué** mide, pero se lee como el total
del proyecto.

**El espacio de trabajo tiene diez crates**, y la suite entera son unas
**561 pruebas** y **22 minutos**.

### El desglose, que dice más que el número

| Qué es | Crates | Pruebas |
|---|---|---|
| **Capa de producción** | `zk-ssl`, `stark-experiment` | **360** |
| Estudio comparativo | `zk-core`, `plonk-experiment`, `halo2-experiment`, `iso-bridge`, `nova-experiment` | 140 |
| ⚠️ **Código de terceros vendorizado** | `ceremony` | **34** |
| ⚠️ **Capa anterior, superada** | `settlement-layer` | **17** |
| Vacíos | `settlement-prover` | 0 |

⚠️ **Las 34 de `ceremony` no son de este proyecto.** Es código vendorizado
de `penumbra-sdk-proof-setup` (Penumbra Labs, MIT/Apache-2.0). Sumarlas al
total sería atribuirse trabajo ajeno.

⚠️ **`settlement-layer` es el predecesor de `zk-ssl`** —este último lo cita
como tal— y **ningún crate depende de él**. Sus 17 pruebas ejercitan código
superado que sigue en el árbol.

### Por qué nadie ejecuta la suite entera

| Crate | Pruebas | Tiempo |
|---|---|---|
| `plonk-experiment` | 36 | **785 s** |
| `halo2-experiment` | 27 | **397 s** |
| `settlement-layer` | 17 | 62 s |
| `iso-bridge` | 3 | 46 s |
| `zk-ssl` | 162 | 27 s |
| `stark-experiment` | 185 | **8 s** |

A los que se suma `zk-core` (Groth16 sobre BLS12-381), con 74 pruebas y
**más de una hora**: sus tests individuales superan el minuto cada uno.

**Los dos crates de producción tardan 35 segundos entre los dos.**

### El tiempo de la suite ES la comparativa

| Paradigma | Crate | Tiempo de sus pruebas |
|---|---|---|
| Groth16 / BLS12-381 | `zk-core` | **> 60 min** |
| PLONK / KZG | `plonk-experiment` | 13 min |
| Halo2 / IPA | `halo2-experiment` | 7 min |
| **STARK / hash** | `stark-experiment` | **8 s** |

La comparativa de rendimiento que este proyecto publica **no hace falta
medirla aparte**: está en cuánto tarda cada crate en probarse a sí mismo.

⚠️ No es una medición controlada —cada crate prueba cosas distintas— pero
el orden de magnitud coincide con las cifras del estudio.

Y ese reparto **es un dato del propio estudio**: confirma la comparativa de
rendimiento que este proyecto publica. PLONK/KZG tarda ~6,9 s por prueba y
los STARK de la capa, milisegundos.

### Cómo se descubrió

⚠️ **No por revisión, sino por accidente.** Al ejecutar `cargo test` sin el
selector `-p`, apareció un resultado de 34 pruebas que no correspondía a
ninguno de los dos crates conocidos. Era `ceremony`, que corre primero por
orden alfabético.

Durante toda la sesión se ejecutaron **dos crates de diez** creyendo que
eran el proyecto entero.

---

## 16. Donde el autor tiene MENOS confianza

Esta es la sección más útil del documento.

### 16.1 `open_account` no exige autorización — **mitigado a medias**

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

### 16.2 La congelación no tiene justificación ni caducidad

**Implementada** con imposición en circuito: la prueba de liquidación
acredita que el emisor no está en el árbol de congelados.

**Lo que queda abierto:**

- El circuito demuestra que dos custodios la autorizaron, **no que
  tuvieran razón**. No hay orden judicial ni motivo registrado.
- **No hay caducidad**: una congelación dura hasta que alguien la levante.
- Una cuenta congelada **sigue pudiendo recibir**. Es deliberado —lo
  contrario dejaría fondos en el limbo— pero merece que un auditor valore
  si encaja con el caso de uso.

### 16.3 Los grados de restricción

**Cinco veces** durante el desarrollo winterfell rechazó un grado mal
declarado. Cada vez se corrigió. La exactitud que exige winterfell hace
improbable que un grado incorrecto pase inadvertido, **pero la
concentración de errores en este punto sugiere revisarlo con atención**.

Especial cuidado con:
- Restricciones que multiplican **dos columnas periódicas** (`C_ACC`).
- Columnas periódicas cuyo periodo real difiere del declarado
  (`circuit_mint`: 8 segmentos × 64 filas llenan la traza y la vuelven
  periódica de periodo 64).

### 16.4 El patrón lockstep

`C_SIBLING` impone que los dos carriles usen el mismo hermano. El
argumento es que eso basta para atar ambas subidas a la misma posición
del árbol.

Está verificado con un test discriminante, **pero el argumento general no
ha sido revisado por nadie más**. Es el hallazgo más original del
proyecto y merece escrutinio.

### 16.5 Los tests negativos

**Tres veces** un test negativo resultó no discriminar: fallaba por una
restricción distinta de la que pretendía probar. Se corrigieron
construyendo testigos internamente coherentes.

**Puede quedar alguno más.** Un auditor debería comprobar, para cada test
negativo, que el testigo corrupto es válido en todo lo demás.

### 16.6 El bloqueo de directorio de `sled` tras cerrar — **hallazgo nuevo**

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

### 16.7 El techo de 63 bits

Las comprobaciones de rango fuerzan el bit más significativo a cero, así
que **ningún valor puede superar 2^63 − 1**.

La capa **no valida** que `max_supply` esté por debajo de ese techo. Con
un tope mayor, las emisiones fallarían con un error confuso en vez de
rechazarse al configurar.

No es una fuga de solidez —los valores fuera de rango se rechazan— pero
sí un fallo de usabilidad que puede confundir un diagnóstico.

### 16.8 El formato de instantánea se queda atrás al añadir estado

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

### 16.9 Colisiones en el árbol de nullifiers

La posición sale de los bits bajos del nullifier. Dos nullifiers pueden
colisionar, y el segundo **no podría gastarse**.

El autor lo clasifica como **denegación de servicio, no ruptura de
solidez**. Ese razonamiento merece verificación independiente: si fuera
incorrecto, sería grave.

---

## 17. Por dónde empezaría el autor si tuviera que romperlo

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

## 18. Limitaciones ya documentadas

No hacen falta descubrirlas; están en `README.md`:

- El operador del nodo es un intermediario de confianza.
- No hay red, consenso, réplicas ni cifrado en reposo.
- La generación de la prueba puede hacerse en el cliente (`client`), pero
  delegarla a un **tercero** exigiría verificar una firma en circuito.
- La resolución IBAN → cuenta está fuera de la prueba.
- El conjunto de gobernanza es inmutable.
- Las cifras de rendimiento son de una sola ejecución.

---

## 19. Cómo reproducir

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

## 20. ⚠️ `cargo test` sin `--release` falla en 56 tests

**Y no porque el código esté mal.** Merece estar aquí porque es lo primero
que ejecutaría quien evalúe el proyecto, y la conclusión natural sería que
está roto.

### La causa

Winterfell comprueba en modo depuración que **el grado declarado de cada
restricción se realice en la traza concreta**:

```
transition constraint degrees didn't match
expected: [..., 2046, 1023]
actual:   [..., 1023,    0]
```

Una restricción booleana como `bit × (bit − 1)` tiene **grado real 0**
cuando ese bit es constante en toda la traza. Ocurre, por ejemplo, cuando el
camino de Merkle de una cuenta tiene todos sus bits a cero — el caso de la
cuenta en la posición 0, que es la que usan casi todos los tests.

### Por qué no es un fallo de solidez

La restricción **sigue imponiendo lo que debe** para los testigos donde el
bit varía. Lo que falla es la comprobación de depuración, que asume que todo
grado declarado se realiza en todo testigo.

### Lo que sí es

⚠️ **Una limitación de reproducibilidad que estaba documentada en un solo
sitio**: un `#[ignore]` dentro de `range_check.rs`. Todos los comandos de la
documentación llevaban `--release`, pero **ninguno decía que fuera
obligatorio**.

Se descubrió porque alguien lo ejecutó sin él. Ahora está en el README,
antes del primer bloque de comandos.

⚠️ **Y significa que la suite en modo depuración no protege**: los 56 tests
que fallan no comprueban nada mientras el modo no se corrija. Cerrarlo
exigiría declarar grados que se realicen en todo testigo, o construir los
tests con posiciones cuyos caminos tengan bits variados. **No está hecho.**

---

## 21. Los códigos ISO, verificados contra el catálogo real

**Se contrastaron los 20 códigos del puente contra
`ExternalStatusReason1Code`**, el catálogo publicado por ISO 20022.
**Tres estaban mal.**

### El grave: `TECH` no existe

| | |
|---|---|
| Se usaba en | **7 variantes de error** |
| En el catálogo | **No aparece** |
| Correcto | `FF10` — *"File or transaction cannot be processed due to technical issues at the bank side"* |

Era el destino de `ProofFailed`, `VerificationFailed`, `Store`,
`AccountLimitReached`, `CustodianSetExhausted`,
`NullifierPositionCollision` y `PendingTreeExhausted`.

⚠️ **Un código inventado que ningún sistema receptor reconocería.** Y venía
del comodín que se eliminó dos secciones antes: quitarlo hizo explícito el
mapeo, pero **no lo hizo correcto**. Hizo falta contrastarlo.

### Los otros dos

| Código | Se usaba para | Qué dice el catálogo | Correcto |
|---|---|---|---|
| `AG01` | Clave de gasto errónea | *"Transaction forbidden on this **type of account**"* | **`AG08`** — *"invalid or missing user or **access right**"* |
| `AM12` | Tope de emisión superado | *"Amount is **invalid or missing**"* | **`AM13`** — *"amount exceeds limits set by **clearing system**"* |

`AG01` es sobre el **tipo de cuenta**, no sobre quién firma. Y el importe no
es inválido cuando se excede un tope: es correcto y excede un límite.

### Los siete que sí eran correctos

`AM04` (fondos insuficientes), `AM02` (importe sobre el máximo), `AC01`
(cuenta inválida), `AC06` (cuenta bloqueada), `AM01` (importe cero), `AM03`
(divisa no admitida), `AM05` (duplicación).

### ⚠️ Dos siguen siendo dudosos, y se dejan como están

| Código | Se usa para | Por qué chirría |
|---|---|---|
| `DS0G` | `StaleState` | El catálogo lo define como *"Signer is not allowed to sign this operation type"*, que no es un estado obsoleto |
| `AM09` | `BalanceOutsideBand` | *"Amount received is not the amount agreed"* — habla del importe recibido, no de un saldo fuera de banda |

**No se cambian porque no hay un candidato claramente mejor**, y sustituir
una elección discutible por otra sin fundamento no mejora nada. Quedan
declarados para que un auditor con conocimiento del estándar decida.

### Los códigos de estado, también verificados

Pertenecen a un catálogo distinto —`ExternalPaymentTransactionStatus1Code`,
antes `TransactionIndividualStatus`— y se contrastaron aparte.

| Código | Nombre | Definición |
|---|---|---|
| `ACSP` | AcceptedSettlementInProcess | Comprobaciones pasadas, liquidación **en curso** |
| `ACSC` | AcceptedSettlementCompleted | Liquidación **completada** en la cuenta del deudor |
| `RJCT` | Rejected | Rechazada |

El ciclo estándar es `RCVD → ACCP → ACSP → ACSC`, así que **la decisión de
§3.11 de `VISION.md` se sostiene**: `send` produce `ACSP` y `claim`
produce `ACSC`.

#### ⚠️ Dos matices que un experto debería revisar

**`ACSC` se define sobre la cuenta del deudor.** En el modelo en dos fases
el deudor **sí** queda debitado tras `send()`, así que por la letra `ACSC`
sería defendible ya ahí. Se descarta porque el acreedor **no tiene el
dinero**, y decírselo a su banco sería engañoso — pero es una lectura, no
una certeza.

**Existe `ACWP` (AcceptedWithoutPosting)**: *"aceptada sin abonar en la
cuenta del acreedor"*. Semánticamente es **lo más cercano** al modelo en dos
fases. Se descarta porque su definición lo acota a retenciones por
escrutinio regulatorio o de fraude, que no es este caso.

⚠️ **Si un experto en el estándar considerara que `ACWP` encaja mejor, el
cambio es de una línea.** Queda declarado para que pueda decidirlo.

### La lección

Se pasó de un mapeo **silencioso** a uno **explícito** hace tres secciones,
y se dio por hecho que explícito implicaba correcto.

> **Hacer visible una decisión no la hace acertada.** Solo permite
> comprobarla — y la comprobación es un paso aparte que hay que dar.

---

## 22. La razón verificar/generar estaba atribuida a la operación equivocada

**Ocho documentos publicaban «verificar cuesta el 0,5-0,8 % de generar».**
La cifra es cierta. **La operación, no.**

### Lo que mide cada cosa

| Operación | Razón | Qué hace al «aplicar» |
|---|---|---|
| **Auditoría** | **0,58 %** | `verify_audit` — **solo verifica** |
| Transferencia | **28,5 %** | `apply` — verifica, muta el árbol y **escribe a disco** |
| Emisión | 58,2 % | idem |
| Destrucción | 63,9 % | idem |

El 0,58 % es el de la **auditoría**, y es el correcto para el argumento que
sostiene: un supervisor comprueba **sin tocar el estado**. La conclusión
—*«podría comprobar millones de operaciones al día»*— se mantiene.

⚠️ **Pero aplicar una transferencia cuesta el 28,5 %**, y eso no es
comparable: incluye escritura a disco.

### Por qué no se había visto

Las demás cifras publicadas **coinciden** con lo medido:

| | Publicado | Medido |
|---|---|---|
| Arranque | 0,67 ms | 1,757 ms (otra máquina) |
| Generar transferencia | ~620 ms | 437,5 ms (otra máquina) |
| Prueba de liquidación | 62 KB | 63.681 B ✅ |
| Mil transferencias | 120,4 MB | 60,7 MB ✅ |

Los tiempos absolutos varían con el hardware y eso estaba declarado. **Una
razón entre dos tiempos de la misma máquina, no.** Ese fue el indicio.

### Cómo se detectó

Ejecutando `cargo test -p zk-ssl --release metrics -- --nocapture` y
comparando con lo publicado.

⚠️ **Es el único hallazgo de toda esta auditoría que exigió medir en vez de
razonar.** Leer el código no lo habría encontrado: el código es correcto, lo
que fallaba era la etiqueta que la documentación le ponía al número.

### Lo que se hizo además

**El tamaño de la prueba se comprueba ahora en el test.** Los tiempos
dependen de la máquina, pero el tamaño **no**: es determinista. Trece
documentos publican 62 KB y 120,4 MB, y hasta ahora nada detectaba que un
cambio los dejara falsos.

---

## 23. La invariante global cambia de forma con dinero en tránsito

**La invariante era `suma de saldos == suministro`.** Con la vía en dos
fases **deja de ser cierta**: el dinero sale de la cuenta del pagador y
espera en un pendiente que no está en ningún saldo.

| | |
|---|---|
| Alice tras enviar 250.000 | 750.000 |
| Bob, que aún no ha cobrado | 50.000 |
| **Suma** | **800.000** |
| Suministro | **1.050.000** |
| **Descuadre** | **250.000** |

⚠️ **El descuadre existía y ningún test lo detectaba.** El que comprueba la
invariante —`total_balances_always_equal_total_supply`— usa `transfer()`,
la vía antigua, que abona al receptor en el acto.

> **Una propiedad que se cree comprobada porque hay un test con ese nombre,
> y el test ejercita otro camino.**

### La forma correcta

```text
suma de saldos + total_pending() == suministro
```

`total_pending()` no existía. Se añadió, con su persistencia: los
compromisos del árbol **no revelan el importe**, así que no puede derivarse
al reiniciar — **hay que guardarlo**. El compilador lo exigió al obligar a
inicializar el campo nuevo en los dos sitios que reconstruyen la capa.

⚠️ **Qué revela**: cuánto hay en tránsito **en total**. No de quién ni para
quién. Es coherente con que el suministro y el tope ya sean escalares
públicos, pero **es información que antes no existía**.

### ⚠️ Y el test estuvo a punto de no comprobar nada

Se escribió **dentro de otra función** por una inserción mal colocada.
Compilaba, no se registraba, y **no ejecutaba**.

Lo delató contar los `#[test]` **declarados frente a los ejecutados**:

```
168 declarados, 167 ejecutados
```

> **Un test que no aparece en la lista es invisible: no falla, no avisa.**
> Solo el recuento lo delata.

Es la misma familia que las cifras rancias de §15: **algo que la
documentación —o el nombre de un test— afirma, y que nadie contrasta**.

---

## 24. Propiedades demostradas sobre un modelo que no se ejecuta

**`crates/zk-ssl/src/pending.rs` es un prototipo del diseño en dos fases.**
De todo lo que contiene, la producción usa **una función**:

| | Quién lo usa |
|---|---|
| `pending_commitment` | ✅ `two_phase.rs` |
| `PendingNotice` | ❌ Nadie fuera del propio fichero |
| `PendingTransfers` | ❌ La capa usa `SparseTree` directamente |

⚠️ **`PendingNotice` está duplicado**: existe en `pending.rs` y en
`two_phase.rs`, **con la misma frase de documentación** y un campo de
diferencia. Es el mismo patrón que el `CustodianPath` duplicado al copiar un
circuito.

### Lo que importa no es el duplicado

Son **los 8 tests** de ese módulo. Demuestran las propiedades del diseño
—que el pagador no necesita el saldo, que nadie más puede cobrar, que el
compromiso no revela al receptor— **sobre un modelo que la capa no
ejecuta**.

Contrastadas una a una con la vía real:

| Propiedad | ¿En producción? |
|---|---|
| El pagador solo necesita la identidad | ✅ |
| El receptor puede cobrar | ✅ |
| Nadie más puede cobrar | ✅ |
| Ni siquiera el pagador lo recupera | ✅ |
| No se cobra dos veces | ✅ |
| El compromiso no revela al receptor | ✅ |
| El pagador sabe **cuándo** se cobra | ⚠️ Fuga residual declarada |
| ⚠️ **Cobrar otro importe se rechaza** | ❌ **No estaba** |

### El hueco encontrado

**Cobrar un importe distinto del comprometido crearía dinero**: el pagador
compromete N y el receptor se abona M.

El circuito lo impide por construcción —el compromiso se forma con
`(identidad, aleatorio, importe)`— pero **nada lo comprobaba en la vía que
se ejecuta**. Se añadió
`circuit_claim::claiming_a_different_amount_is_rejected`.

> **Una propiedad de seguridad demostrada sobre un modelo no está demostrada
> sobre lo que se ejecuta.**

### ⚠️ Y esto no se limita a este módulo

`stark-experiment` tiene **11 módulos del estudio comparativo** que nadie
usa (§15), y `crates/settlement-layer` es una capa entera superada (§11).
**Sus tests demuestran propiedades de código que no se ejecuta.**

No es un defecto —documentan cómo se llegó al diseño— pero **contar sus
tests como garantías del sistema sería falso**, y quien lea sus nombres
puede creerlo.

---

## 25. ⚠️ Al sustituir una vía por otra se perdió el límite regulatorio

**Es el hallazgo más serio de esta auditoría**, y salió de repetir con
`crates/settlement-layer` el ejercicio de §24.

### Lo que tenía la capa superada

`settlement_with_foreign_limit_is_rejected` manipulaba el límite declarado
en el recibo y comprobaba que `apply` lo rechazara:

> *El límite lo impone el **sistema**, no quien transfiere.*

### Lo que tenía la vía en producción

| | Límite regulatorio |
|---|---|
| `circuit_settlement` + `apply` | ✅ Entrada pública **y** comprobación al aplicar |
| `circuit_send` + `apply_send` | ❌ **Ninguna de las dos** |

⚠️ **El límite quedaba impuesto solo en `send()`, al generar.** Eso ata a
quien use esa función, pero **quien construya su propia traza y su propia
prueba puede llamar directamente a `apply_send`**.

Y como la vía en dos fases es ahora **la única de ISO** (§3.11 de
`VISION.md`), el límite regulatorio del sistema no se imponía por ningún
camino verificable.

### Corregido, y hasta dónde

✅ `apply_send` comprueba el importe **probado** —`pi.amount`, no el
parámetro— contra el límite del sistema.
✅ `a_send_declaring_more_than_the_limit_is_rejected` lo demuestra
manipulando el importe declarado, igual que hacía el test de la capa
superada.

### ✅ Cerrada del todo: el límite vuelve al circuito

`circuit_send` lleva ahora `regulatory_limit` como **entrada pública**, y un
quinto segmento de rango descompone `límite − importe` en 63 bits. Si el
importe lo superara, esa resta envuelve y no cabe.

**Son dos comprobaciones que se componen:**

| Dónde | Qué prueba |
|---|---|
| El circuito | `importe ≤ límite declarado` |
| La capa | El límite declarado **es el del sistema** |

Juntas dan `importe ≤ límite del sistema`, y —a diferencia de una
comprobación solo de capa— **un tercero con la prueba puede verificar la
primera mitad**. Es la misma composición que tenía la vía antigua.

**Tres tests lo fijan**: el par en el circuito
—`sending_more_than_the_regulatory_limit_is_rejected` y
`..._exactly_..._is_allowed`— y
`a_send_declaring_more_than_the_limit_is_rejected` en la capa.

⚠️ **El ataque cambió de forma al cerrarlo.** Antes bastaba con declarar más
importe; ahora eso no da prueba válida, y el ataque real es **declarar un
límite enorme**. El test de capa se reescribió para eso.

### ⚠️ Cuatro fallos propios al implementarlo

| Fallo | Quién lo cazó |
|---|---|
| `NUM_SEGMENTS` a 5 sin añadir el quinto valor | Los tests **positivos** |
| `COL_LIMIT` declarada sin rellenar | `check_columns.py` |
| Cuenta de aserciones sin actualizar (41 → 42) | `prove`, **pero solo tras conservar el mensaje del pánico** |
| El ayudante del test **solo probaba, no verificaba** | El test negativo, que no rechazaba |

⚠️ **El tercero costó tres rondas** porque el ayudante descartaba el mensaje
con `Err(_) => "prove hizo panic"`. Winterfell decía *«expected 41
assertions, received 42»* y **ese texto se estaba tirando**.

✅ **Corregido en los nueve ayudantes que lo hacían.** Los diez devuelven
`Result<(), String>`, así que el cambio encajaba en todos —comprobado antes
de aplicarlo, no después—.

Quedan tres con `Err(_) => {}` —`circuit_settlement`, `compliance_circuit`,
`dual_climb`— que **descartan a propósito**: su lógica es *«si hubo pánico,
se detectó»* y ahí el mensaje no aporta.

⚠️ **Y el cuarto es el más instructivo**: en modo release winterfell **no
comprueba las restricciones al generar**. Un test que solo llame a `prove`
**no comprueba nada**.

### `apply_claim` y `apply_mint_to_pending` tampoco lo comprueban

Y **probablemente no deban**: al cobrar, el importe ya pasó el límite al
enviarse; al emitir, lo que aplica es el tope de suministro, que sí se
comprueba. **No se han tocado**, y queda dicho para que un auditor lo
confirme en vez de suponerlo.

### La cadena que llevó hasta aquí

| | |
|---|---|
| Un tipo `PendingNotice` duplicado | Anotado con **nueve rondas de retraso** |
| Al anotarlo | `pending.rs` resultó ser un prototipo que no se ejecuta |
| Contrastar sus 8 tests | Faltaba `claiming_a_different_amount_is_rejected` (§24) |
| Repetir con los 17 de `settlement-layer` | **Esta regresión** |

> **Contrastar lo que un test demuestra con dónde se ejecuta** encontró más
> que las dos herramientas automáticas de §12 y §14 juntas.

---

## 26. ⚠️ La vía en dos fases no dejaba rastro en el registro

**`two_phase.rs` era el único módulo que no registraba nada.**

| Módulo | ¿Registra? |
|---|---|
| `accounts`, `burn`, `freeze`, `governance`, `mint`, `recovery`, `transfer` | ✅ |
| **`two_phase`** — envío, cobro, emisión a pendiente | ❌ |

El registro de transiciones es **el mecanismo de auditoría del sistema**: una
cadena donde cada entrada parte de donde acabó la anterior, y `verify`
comprueba que no falte ninguna.

⚠️ **Y como la vía en dos fases es ahora la única de ISO (§25), los pagos de
un banco no quedaban registrados.**

### Cómo se descubrió

Migrando `the_log_chains_every_operation` de `transfer` a `send`/`claim`.

Se predijo que el recuento **subiría de 5 a 6** —una entrada más, porque son
dos operaciones— y **el test midió 4**.

> **Si la predicción hubiera acertado por casualidad, el hueco seguiría ahí.**
> Lo delató la diferencia, no el acierto.

### Corregido

Tres tipos de operación nuevos —`Send`, `Claim`, `MintToPending`— y su
registro en las tres funciones de aplicación, **antes de persistir**, igual
que las demás: si el proceso muere en medio, el lote atómico incluye o
excluye las dos cosas.

⚠️ **Con un coste declarado en `MintToPending`.** El registro encadena la
raíz de **cuentas**, y una emisión a un pendiente no la toca: su entrada
declara la misma raíz en los dos lados. Quien lea el registro ve que hubo
una emisión, pero **la raíz no le dice cuál**. Encadenar también la de
pendientes exigiría cambiar el formato del registro, y **no está hecho**.

### El patrón, por segunda vez

Es el mismo que §25: **al sustituir una vía por otra se perdió algo que la
primera hacía**, y nadie lo notó porque los tests de la vía nueva se
escribieron mirando la vía nueva.

| | Lo que se perdió |
|---|---|
| §25 | El límite regulatorio impuesto en el circuito |
| §26 | El rastro en el registro de auditoría |

> **Sustituir no es solo escribir lo nuevo: es contrastar lo que hacía lo
> viejo.**

---

## 27. Once tests de reinicio comparaban valores; uno solo intentaba el ataque

Al migrar los tests de nullificadores apareció una asimetría:

| | |
|---|---|
| Comparar una raíz antes y después de reiniciar | Un **indicio** |
| Intentar la operación prohibida tras reiniciar | La **propiedad** |

**De los doce tests de reinicio, once hacían lo primero.**

No están mal: comprueban que el estado se restaure. Pero **ninguno demuestra
que restaurarlo baste** para que el ataque siga bloqueado.

### Lo que encontró el que sí lo intenta

`a_restart_does_not_allow_claiming_twice` intenta cobrar dos veces un
pendiente después de reiniciar.

✅ **El doble cobro estaba bloqueado.** El mecanismo original es correcto:
al cobrar, la capa escribe la hoja **vacía** en el mismo lote atómico, y al
cargar un digest cero elimina la hoja.

⚠️ **Pero destapó un fallo introducido en esta misma auditoría.**
`total_pending()` —añadido para la invariante de §23— persistía los importes
en un bucle aparte que escribía **solo los actuales**, y por tanto **nunca
borraba los de los pendientes ya cobrados**. Tras reiniciar, el dinero en
tránsito se contaba de más.

**No creaba dinero**: la invariante quedaba mal, no el saldo.

### La lección

El mecanismo correcto estaba **al lado**, en el mismo fichero, y no se
copió:

| | Al consumir |
|---|---|
| `pend:` (original) | Escribe el valor **vacío**; al cargar, el cero elimina |
| `pamt:` (añadido aquí) | Escribía solo los vivos; **el muerto se quedaba** |

> **Añadir persistencia nueva sin copiar el mecanismo que ya funciona al
> lado** es el mismo error que inventar una función que no existe, y lo
> comete quien escribe deprisa por analogía en vez de leer.

### ✅ Los doce revisados

| Cómo estaban | Cuántos |
|---|---|
| Ya intentaban la operación | 4 |
| Se les añadió el ataque | 5 |
| Migrados a la vía en dos fases | 1 |
| Test nuevo escrito para verlo fallar | 1 |

**Un fallo real**: el máximo del cupo de custodios no se persistía (§28).

⚠️ **Y dos correcciones a la clasificación previa.**
`the_account_counter_survives_restart` y `pending_transfers_survive_restart`
figuraban entre los que «solo comparan valores» y **ya intentaban la
operación**: el primero abre una cuenta y comprueba que no pise a las
existentes; el segundo cobra el pendiente y lo aplica.

> **Clasificar por lo que un test parece hacer, en vez de leerlo, produjo dos
> falsos positivos de once.** El coste fue bajo aquí; en una lista de
> hallazgos habría sido reportar defectos inexistentes.

### Los ataques que se añadieron

| Test | Lo que ahora intenta |
|---|---|
| Congelación | Que una cuenta congelada **gaste** |
| Suministro | **Pasarse del tope**, y llegar justo |
| Custodios | Que un conjunto **revocado** emita |
| Gobernanza | Que un **no-gobernador** cambie el conjunto |
| Recuperación | **Reaplicar** una recuperación ya aplicada |
| Ledger general | **Operar** con el estado recuperado |

Ese último merece nota. Los saldos restaurados dicen que el estado se leyó;
no dicen que se leyera **entero**. Una operación posterior falla si falta el
nonce, la raíz de congelados o el contador de cuentas — campos que ninguna
aserción de saldo tocaría.

---

## 28. ⚠️ Reiniciar el nodo renovaba el cupo de custodios

**Es el hallazgo del ejercicio que §27 proponía**, y apareció al tercer test
revisado de los once.

### El cupo son dos cosas

| | ¿Persistía? |
|---|---|
| `custodian_uses` — el contador | ✅ `meta:cust_uses` |
| **`max_custodian_uses` — el máximo** | ❌ **No**: volvía al valor por defecto |

⚠️ **Quien hubiera restringido el cupo —para limitar a un conjunto de
custodios bajo sospecha— veía la restricción levantada por un reinicio.**

`the_custodian_quota_survives_restart` comprobaba que **el contador**
sobreviviera. El cupo son dos valores, y **solo uno estaba en la aserción**.

### Corregido

`meta:cust_max` se escribe con el contador, en el mismo lote atómico, y se
restaura al cargar. Un ledger anterior a este campo supone el valor por
defecto, que es exactamente el que estaba usando.

`a_restart_does_not_renew_an_exhausted_custodian_quota` lo demuestra:
agota el cupo, reinicia, e **intenta aplicar una emisión**.

### ⚠️ Dos errores propios al escribirlo

**Primero, atacó el paso equivocado.** El cupo se consume en `apply_mint`,
no en `mint` —decisión documentada: una prueba generada y no aplicada no
debe gastar cupo— y el test comprobaba `mint()`.

⚠️ **Ocurrió cuatro veces** en esta auditoría, sobre cuatro operaciones
distintas:

| Test | Atacó | Debía atacar |
|---|---|---|
| Cobro doble tras reinicio | `claim()` | `apply_claim()` |
| Cupo agotado tras reinicio | `mint()` | `apply_mint()` |
| Custodios revocados | *(comprobado antes de escribir)* | `apply_mint()` |
| Gobernanza tras reinicio | `update_custodians()` | `apply_governance()` |

> **La capa separa generar de aplicar en todas sus operaciones.** Un test que
> ataca la generación **no comprueba nada**, y esa separación está documentada
> y tiene su propio test.

⚠️ **Y el patrón correcto estaba en el mismo fichero desde el principio.**
`a_custodian_cannot_change_the_custodian_set` genera, aplica, y comprueba que
**aplicar** falle. Cuatro tests se escribieron mal con ese ejemplo a la vista.

**La comprobación que lo habría evitado cuesta cinco líneas**: barrer el
fichero buscando una aserción de fallo sobre una generación sin aplicar
después. Se hizo al cuarto intento y confirmó que no quedaba ninguno más.

> **Buscar el patrón malo en todo el fichero, en vez de arreglar la aparición
> que se ve.** Es lo que `check_figures.py` hace con las cifras, y no se
> aplicó aquí hasta tarde.

**Segundo, la clasificación previa era falsa.**
`the_account_counter_survives_restart` figuraba entre los once que «solo
comparan valores» y **sí intenta el ataque**: abre una cuenta nueva y
comprueba que no pise a las existentes.

⚠️ **Quedan ocho sin revisar.**

---

## 29. ⚠️ Una cuenta congelada ya no puede recibir: recibe hacia el limbo

**La propiedad no se quitó. Cambió de significado al cambiar quién actúa.**

`freeze.rs` documenta la decisión original:

> *«**Recibir.** Una cuenta congelada no puede gastar, pero sí seguir
> recibiendo. Impedirlo exigiría comprobar también al receptor y **dejaría
> fondos en el limbo**.»*

En la vía de un paso, recibir era **pasivo**: el pagador actualizaba las dos
hojas y el receptor no hacía nada. En la vía en dos fases, **cobrar es una
acción del receptor**, y tanto `claim` como `circuit_claim` —que lleva
`frozen_root`— la rechazan si está congelado.

### La secuencia real

| Paso | ¿Funciona? |
|---|---|
| Enviar a una cuenta congelada | ✅ |
| El dinero sale del pagador al pendiente | ✅ |
| Que la congelada lo cobre | ❌ `AccountFrozen` |

⚠️ **El dinero queda exactamente en el limbo que la decisión original quería
evitar**: salió del pagador, no llegó al receptor, y solo se libera si alguien
levanta la congelación.

Lo fija `a_frozen_account_receives_into_limbo`, que antes se llamaba
`a_frozen_account_can_still_receive` y afirmaba lo contrario.

### Decisión pendiente, y no es técnica

**¿Debe una cuenta congelada poder cobrar lo que ya le enviaron?**

| A favor de permitirlo | A favor de impedirlo |
|---|---|
| El dinero ya salió; retenerlo no protege a nadie | Cobrar es una operación del titular, y está intervenido |
| Evita el limbo que el diseño rechazaba | El importe seguiría inmovilizado igual, pero en su cuenta |
| El pagador no eligió congelar a nadie | Un cobro cambia la raíz de cuentas |

Implementarlo exigiría **quitar `frozen_root` de `circuit_claim`**, y eso es
un cambio de circuito con su propio par de tests. **No está hecho, y la
decisión no es del implementador.**

### Cómo apareció

Migrando `a_frozen_account_can_still_receive` al ayudante de dos fases. El
test falló con `AccountFrozen(1)` —el índice del **receptor**— y esa
identidad fue lo que delató el cambio de propiedad.

> **Migrar un test no es traducirlo: es volver a preguntar si lo que afirmaba
> sigue siendo cierto.**

---

## 30. ⚠️ Enviar a un identificador inexistente pierde el dinero

**No es un defecto de implementación. Es un coste del modelo que no estaba
declarado.**

| Vía | A quién se paga | Si no existe |
|---|---|---|
| `transfer` | Un **índice** de cuenta | `AccountNotFound` |
| `send` | Un **identificador público** | **Funciona. El dinero se pierde.** |

`send` no puede comprobar que alguien tenga ese identificador **sin revelar
quién está en el árbol**, que es justo lo que el diseño evita. Así que no lo
comprueba.

El envío se aplica, el dinero sale del pagador, y queda en un pendiente que
**nadie puede cobrar jamás**. Un dígito mal en el identificador pierde el pago
**sin ningún aviso**.

⚠️ **No hay devolución.** El importe queda fuera de circulación sin dejar de
contar en el suministro: la invariante global se cumple —`saldos + pendientes
== suministro`— y el dinero es inalcanzable.

Lo fija `sending_to_a_nonexistent_recipient_loses_the_money`, que antes se
llamaba `transferring_to_a_nonexistent_account` y afirmaba lo contrario.

### Qué haría falta, y su coste

| Opción | Coste |
|---|---|
| Comprobar la existencia en la capa | Revela quién tiene cuenta: **rompe el modelo** |
| Devolución tras un plazo | Exige tiempo en el circuito, que no existe |
| Confirmación fuera de banda antes de enviar | No es criptografía, es producto |

**Ninguna está implementada**, y la primera es incompatible con el diseño.

### El patrón, por segunda vez en dos rondas

| | Lo que cambió |
|---|---|
| §29 | Una congelada recibe hacia un limbo |
| §30 | Un identificador inexistente pierde el dinero |

Los dos vienen de lo mismo: **la vía en dos fases mueve la acción al
receptor**, y las propiedades que dependían de que el pagador lo hiciera todo
dejaron de cumplirse. Los dos aparecieron **migrando tests**, no revisando
código.

> **Un cambio de arquitectura no invalida los tests uno a uno. Invalida la
> clase de afirmación que podían hacer.**

---

## 31. ⚠️ Las cifras publicadas miden una operación retirada

**Un pago son dos pruebas, no una.** El arnés medía `transfer` —la vía de un
paso— y esas cifras están en cinco documentos y en un DOI.

| | Publicado | Medido (pago completo) |
|---|---|---|
| Acumulación por mil pagos | 120,4 MB | **120,4 MB** |
| Aplicar / generar | 28,5 % | **28,5 %** |
| Tamaño de una prueba | ~62 KB | **~62 KB, sin cambio** |

Lo que **no** cambió es el tamaño de cada prueba. Lo que cambió es que hacen
falta dos: `send` y `claim`.

> **La cifra vieja no era un error de medición: medía una operación que dejó
> de ser la de producción.**

### El desglose, que aporta algo nuevo

| Fase | Generar | Aplicar | Prueba |
|---|---|---|---|
| Envío | 282,9 ms | 129,3 ms | 63.168 B |
| **Cobro** | **500,7 ms** | 94,0 ms | 63.086 B |

⚠️ **Cobrar cuesta casi el doble que enviar.** El circuito de cobro recorre
el árbol de pendientes **y** el de cuentas, mientras que el de envío toca una
cuenta y añade un pendiente.

Para un sistema de pagos esto importa: **el trabajo caro recae en quien
recibe**, no en quien paga.

### Lo detectó una guarda del propio arnés

`metrics.rs` llevaba una comprobación con este texto:

> *«...si el tamaño ha cambiado de orden, esas cifras son falsas»*

Saltó en cuanto la medición pasó a la vía nueva. **Hizo exactamente su
trabajo**, y sin ella el cambio habría pasado en silencio dejando cinco
documentos y un DOI con cifras que no describen nada que se ejecute.

Ahora comprueba **tres** cosas: cada fase por separado y el pago completo. Si
una engordara y la otra adelgazara, la suma podría cuadrar y la guarda callar.

### Lo que exige del material publicado

El preprint tiene DOI
[10.5281/zenodo.21677737](https://doi.org/10.5281/zenodo.21677737). Zenodo
permite versiones nuevas que heredan el *concept DOI*, así que la corrección
no rompe ninguna cita.

⚠️ **Y es coherente con lo que el propio paper declara.** Su §7.5 ya dice que
una versión anterior describía la acumulación como el límite real del sistema
y que eso era incorrecto. Esta corrección es de la misma clase: **el número
describía otra cosa de la que se creía.**

---

## 32. ⚠️ Retirar `transfer()` no cierra el límite del cumpleaños

**Esta sección corrige una afirmación repetida durante toda la auditoría.**

El plan era: migrar las 35 llamadas a `transfer()`, retirar la función, y con
ella el árbol de nullificadores y el límite de ~65.000 pagos de §13. Las 35
llamadas están migradas y no queda ningún uso.

⚠️ **Pero hay una segunda vía, y sigue viva.**

`client.rs` expone `transfer_materials` → `prove_transfer` → `apply`: un
cliente por compromisos completo, **construido sobre `circuit_settlement`**,
que calcula su nullificador localmente y lo entrega a la capa.

| | Vía | ¿Nullificador? |
|---|---|---|
| Capa | `transfer` + `apply` | Sí — **sin usos, retirable** |
| **Cliente** | `prove_transfer` + `apply` | **Sí — API pública, viva** |
| Dos fases | `send` + `claim` | No |

**Retirar `transfer()` deja `apply()` en pie**, porque el cliente la necesita,
y con ella el árbol de nullificadores y su límite.

### Por qué se pasó por alto

El plan se formuló contando **llamadas a `transfer()`**, y la vía del cliente
no llama a `transfer()`: **construye su propia traza**. Un barrido por nombre
de función nunca la habría encontrado.

> **Contar llamadas a una función no mide cuánta superficie depende de lo que
> esa función usa.** Lo que había que contar era el uso del circuito, no el
> de la función.

### Qué haría falta de verdad

`client.rs` es el modelo por compromisos para la vía de un paso, y `send` /
`claim` **ya son ese mismo modelo** —también reciben el estado del titular—.
Migrarlo significa dar al cliente un equivalente de `prove_transfer` sobre
`circuit_send`, con sus materiales y su cobro.

⚠️ **No está hecho, y es más trabajo que las 35 llamadas juntas**: es una API
pública con su propia documentación, sus tests y su lugar en los papers.

### Lo que sí se ganó

Las 35 migraciones **no fueron en balde**: produjeron cuatro hallazgos —§25,
§26, §29, §30— y dejaron la vía de dos fases ejercitada por toda la suite.
Pero **el objetivo declarado no se alcanzó**, y decirlo importa más que el
progreso.

---

## 33. ⚠️ La vía de producción no demuestra la separación de claves

**Es la propiedad que los papers citan primero, y la vía que se ejecuta no la
enseña.**

`PAPER.md` §3 dice: *«La clave de gasto no sale de la máquina del cliente»*.
Los tres preprints la repiten, y es el argumento institucional central: el
operador procesa sin custodiar claves.

### Cómo la demuestra cada vía

| Vía | Forma | ¿Enseña la separación? |
|---|---|---|
| Un paso | `transfer_materials` (capa) → **`prove_transfer`** (función libre, con la clave) | ✅ La clave nunca toca la capa |
| **Dos fases** | **`layer.send(clave, ...)`** | ❌ **Método de la capa, con la clave** |

`client.rs` existe precisamente para enseñarla: pide **materiales** —caminos,
raíces, límite— y prueba en una función libre que no conoce la capa.

⚠️ **`circuit_send` no tiene equivalente.** La única forma de generar un envío
es llamar a un método de `SovereignLayer` pasándole la clave de gasto.

### La propiedad puede seguir siendo cierta; lo que falta es enseñarla

En un despliegue real el cliente podría ejecutar el código de la capa en su
máquina, y la clave no viajaría a ninguna parte. **La forma de la API no lo
impide.**

Pero tampoco lo demuestra, y `client.rs` se escribió justo para eso. **Un
lector que quiera comprobar la afirmación central de los papers solo encuentra
la demostración en la vía retirada.**

### Es separable, y se ha comprobado

`send` lee de la capa: caminos de cuentas y pendientes, raíces de congelados,
opciones, límite regulatorio, suministro y la posición libre para el
pendiente.

**Todo eso son datos públicos o derivables**, del mismo tipo que
`TransferMaterials` ya entrega. La separación no es imposible: **no está
hecha**.

### Lo que exigiría

| Pieza | Equivalente existente |
|---|---|
| `send_materials(...)` | `transfer_materials` |
| `prove_send(materials, clave)` | `prove_transfer` |
| `claim_materials` / `prove_claim` | **Sin equivalente**: el cobro no existía |

⚠️ **La tercera fila no tiene precedente.** El cobro es una operación del
receptor que la vía de un paso no tenía, así que su lado cliente hay que
diseñarlo, no traducirlo.

---

## 34. Qué NO demuestra este documento

Que el sistema sea seguro. Demuestra que **el autor ha buscado sus
propios fallos de forma sistemática y ha encontrado algunos**, incluidos
dos al escribir estas páginas.

Es exactamente por eso que hace falta que lo mire alguien más.
