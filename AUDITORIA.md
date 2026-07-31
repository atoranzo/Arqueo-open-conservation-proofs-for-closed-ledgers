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
| `circuit_send` — debita y crea el pendiente | ✅ 16 tests |
| `circuit_claim` — demuestra que es suyo y cobra | ✅ 15 tests |
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
| **`circuit_send`** — el pagador debita y crea el pendiente | ✅ **16 tests** |
| **`circuit_claim`** — el receptor demuestra que es suyo y cobra | ✅ **15 tests** |
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

~~Aplicada a **12 circuitos**. Los doce, **limpios**: toda
restricción declarada reacciona a alguna perturbación del testigo.~~

⚠️ **Corregido después (§20): la cobertura real era ONCE.** El informe de
`circuit_audit` se generó sobre una traza de referencia inválida y no
valía; el `12 passed` de abajo cuenta tests que corrieron, no informes
válidos. El párrafo se conserva tachado porque así se publicó.

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

~~Doce circuitos, ninguna restricción vacía.~~ **Once** (§20). La herramienta está validada
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

La documentación afirma **373 pruebas ejecutables** y da los dos comandos
que las ejecutan. Es preciso sobre **qué** mide, pero se lee como el total
del proyecto.

**El espacio de trabajo tiene diez crates**, y la suite entera son unas
**565 pruebas** y **22 minutos**.

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
- Una cuenta congelada **sigue pudiendo recibir**. Era deliberado en la
  vía de un paso —lo contrario dejaría fondos en el limbo—, pero ⚠️ **con
  la retirada de esa vía (§36) el argumento se dio la vuelta**: en dos
  fases cobrar es una acción del receptor, y `circuit_claim` la rechaza si
  está congelado, así que el dinero enviado a una cuenta congelada **queda
  en el limbo que la decisión original quería evitar** (§7). El diseño no
  se ha readaptado a ese cambio. Un auditor debe valorarlo con el
  comportamiento real, no con el original.

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
- ~~No hay red, consenso, réplicas ni cifrado en reposo.~~
  ⚠️ **Rancio (30-07-2026): sí hay cifrado en reposo.** `persistence` tiene
  `open_encrypted` y sella cada valor con la clave; las instantáneas se
  cifran y una copia cifrada no se importa sin clave. El README ya no lo
  lista como carencia; esta sección se quedó atrás. Lo cierto sigue siendo:
  **no hay red, consenso ni réplicas**.
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
cargo test -p zk-ssl --release              # la capa, 174 tests
cargo test -p stark-experiment --release    # los doce circuitos, 201 tests
cargo test -p zk-ssl --release metrics -- --nocapture
```

**Los tests de circuito conviene ejecutarlos también en debug**: winterfell
valida las restricciones al generar solo en ese modo, y da el índice y la
fila exactos del fallo.

---

## 20. ⚠️ `cargo test` sin `--release` falla en 65 tests de la capa

> **⚠️ CORRECCIÓN DE UNA CORRECCIÓN.** Una revisión anterior afirmó que esta
> sección exageraba —«decía 56 y son 2»— tras medir **`stark-experiment`**.
> Los 56 eran de **`zk-ssl`**, otro crate. La cifra no estaba rancia: estaba
> **bien**, y hoy son **65** porque la suite ha crecido.
>
> **Se midió algo parecido a lo afirmado y se dio por lo mismo**, que es
> exactamente el error que esta auditoría documenta en otros sitios. Se deja
> escrito porque el fallo es más instructivo que la cifra.

### El estado, por crate

| Crate y modo | Resultado |
|---|---|
| `stark-experiment` en depuración | ✅ **199 pasan**, 2 ignorados |
| `stark-experiment --release` | ✅ **201 pasan**, **0 ignorados** |
| **`zk-ssl` en depuración** | ❌ **107 pasan, 65 fallan** |
| `zk-ssl --release` | ✅ **174 pasan** |

**Reparto de los 65**: 48 en `tests`, 9 en `iso`, 4 en `snapshot`, 3 en
`metrics`, 1 en `client`.

### El limite, declarado (entrada 6/24/25, decidido en §46)

Estos 65 fallos **no son un defecto de solidez, y no se van a arreglar**.
La razon, y por que esa decision es la correcta:

Winterfell comprueba en depuracion que el grado declarado de cada
restriccion se **realice** en la traza concreta que se prueba. Una
restriccion cuyo grado **depende del valor del testigo** viola esa
comprobacion en los testigos donde el grado colapsa, aunque sea
perfectamente valida. En esta capa eso ocurre en dos familias:

- **Bits de camino de Merkle** (arboles de cuentas, pendientes y
  congelados). Una posicion baja tiene los bits altos a cero, y una
  restriccion booleana `bit × (bit − 1)` sobre una columna constante-cero
  tiene grado cero en vez del declarado (§35, §37.7).
- **Margenes que pueden ser cero** por diseno: el margen del tope de
  emision cuando se emite exactamente hasta el tope, y la diferencia de la
  comprobacion de rango cuando `amount == balance`, que el circuito de
  cumplimiento **necesita** (§37.2, caso B).

**En release —el modo de produccion— winterfell no comprueba grados**, y las
pruebas se generan y verifican correctamente en ambos casos. La comprobacion
de grados en depuracion es una red *adicional*, util donde aplica, y esta
clase de restriccion queda fuera de su alcance por naturaleza, no por un
fallo del circuito.

**Por que se declara y no se corrige** (§46): la unica forma de que los bits
de camino no colapsen es no asignar las posiciones bajas, y como
`allocate_pending` reutiliza huecos (§46.1), eso obliga a **migrar los
pendientes vivos** de los ledgers existentes —mover valor en transito, del
peso de §36— para arreglar una comprobacion que el modo de produccion no
necesita. Es desproporcionado. Y para los margenes de dominio (§37.2 caso B)
no hay arreglo posible: el valor cero es legitimo.

> **Limite conocido, no fallo.** El proyecto usa release como modo de
> produccion y lo documenta. Perseguir el 100 % de tests en depuracion
> costaria una migracion de fondos o seria imposible, y compraria una
> comprobacion redundante con release. Se elige coherencia sobre
> completitud.

### ⚠️ `check_figures.py` comprobaba tres cifras, no todas

Toda esta auditoría se ha apoyado en esa herramienta para cada actualización
de cifras, y su salida —*«22 documentos: todas las cifras coinciden con el
código»*— **significaba otra cosa**.

Comprobaba **tres**:

| Cifra atribuida | Valor |
|---|---|
| `settlement-layer` | 17 |
| `circuit_threshold` | 11 |
| `circuit_mint_pending` | 16 |

⚠️ **Ninguna es una cuenta principal.** Las de `zk-ssl` y `stark-experiment`
—las que cambian en cada ronda— **nunca se comprobaron**.

### Por qué

Los patrones exigen el nombre del crate a menos de 25 caracteres del número.
Una línea como:

```
cargo test -p zk-ssl --release        # la capa: 174 tests
```

tiene cuarenta de separación. La cifra no se atribuye, y la herramienta
**la ignoraba en silencio**.

### Cómo se encontró

Aplicando a `check_figures` el mismo criterio que se acababa de aplicar a
`check_tests`: **probarla en el sentido negativo**. Se puso `999 tests` en el
`README` y siguió diciendo que todo coincidía.

> **Es la segunda herramienta de auditoría rota que aparece hoy**, después
> del detector de restricciones vacías trabajando sobre una traza inválida
> (§20). Las dos por lo mismo: **nadie las había probado contra un caso que
> debieran rechazar**.

### Corregido: declara su cobertura

Ahora informa de **cuántas cifras ha atribuido** y lista las que hablan de
tests y no ha podido comprobar —34 en el momento de escribir esto—.

> **Una herramienta que no declara su cobertura no dice nada.** «Todo
> correcto» sobre tres cifras de treinta y siete es indistinguible de «todo
> correcto» sobre ninguna.

⚠️ **No se han reescrito los documentos** para que el crate quede pegado al
número. La herramienta ya no engaña sobre lo que mira; ampliar su alcance es
una decisión aparte, con su coste en frases reescritas.

### ✅ `tools/check_tests.py`

Las dos formas en que un test puede no proteger han aparecido **una vez cada
una** en este proyecto:

| Forma | Dónde apareció |
|---|---|
| `#[test]` anidado en una función | `balances_plus_pending` (§17) |
| `#[ignore]` sin condición | `zero_value_only_works_in_release_mode` (§20) |

Las dos compilan. Las dos pasan desapercibidas. Ninguna falla ni avisa.

La herramienta barre los diez crates y busca las dos. **533 tests
declarados, ninguno de los dos casos.** Está validada en los dos sentidos:
introducir un `#[ignore]` a propósito la hace fallar, y retirarlo la
devuelve a verde.

> **Un fallo que ha ocurrido una vez puede volver a ocurrir en silencio.**
> Convertir la comprobación puntual en herramienta cuesta menos que
> encontrarlo dos veces.

### ⚠️ Un test que nadie ejecutaba

`range_check::zero_value_only_works_in_release_mode` llevaba `#[ignore]`
**sin condición**: no se ejecutaba tampoco en release, donde funciona. Su
documentación decía cómo lanzarlo a mano con `-- --ignored`.

> **Un test que depende de que alguien recuerde ejecutarlo no protege nada.**

Y no era menor: comprueba `amount == balance` —que produce `diff = 0`—, un
caso que **el circuito de cumplimiento necesita** y que Groth16 y Halo2
verifican explícitamente.

✅ Ahora usa `#[cfg_attr(debug_assertions, ignore)]`: se salta solo donde el
problema existe, y **release lo ejecuta con el resto**.

### Lo que sí valió de aquella revisión

Aunque la conclusión era falsa, los dos fallos que encontró en
`stark-experiment` eran **reales** y están corregidos:

### Fallo 1 — el test de mutación trabajaba sobre una traza inválida

`circuit_audit::no_constraint_is_vacuous` construía su traza de referencia
con **saldo 1.000.000 y banda [700.000, 800.000]**: el saldo **fuera de la
banda**. La restricción que detecta eso saltaba, haciendo su trabajo.

⚠️ **Y en `--release` no se notaba.** `buscar_vacias` lleva un
`debug_assert` que comprueba que la referencia sea válida, y en release **no
se ejecuta**: la herramienta seguía adelante marcando como «disparadas»
restricciones que **ya lo estaban antes de perturbar nada**.

**Su informe para ese circuito no valía.** La afirmación publicada —*«prueba
por mutación: 12 de 12 circuitos limpios»*— cubría once.

⚠️ **Y «de producción» ya no es exacto.** De esos doce, `circuit_settlement`
y `nullifier_tree` pertenecen a la **vía retirada** (§32): quedan **diez de
producción** más esos dos, que ~~se conservan por compatibilidad de
formato~~ se conservan porque **documentan la vía comparada en los
papers** —la compatibilidad de formato dejó de ser motivo al retirarse el
árbol con migración verificada (§36)—.

> **La herramienta tenía una autocomprobación que nadie había ejecutado,
> porque toda la documentación decía `--release`.**

Los otros once sí estaban bien: el mismo `debug_assert` los valida a todos y
**solo saltó en `circuit_audit`**. Esa ejecución es el barrido, y es más
fuerte que cualquier patrón de búsqueda.

### Fallo 2 — el margen del tope es cero, y un grado con él

`circuit_mint_pending::minting_exactly_up_to_the_cap_is_allowed` alcanza el
tope **exactamente**, así que `tope − suministro_nuevo` vale **cero** y los
63 bits del segmento de rango son todos cero. Las restricciones booleanas
sobre ellos tienen grado real **0**.

✅ **Se salta en depuración con `#[cfg_attr(debug_assertions, ignore)]`**, y
el motivo va en el propio test.

⚠️ **No se ha debilitado el test.** Probar con margen 1 en vez de 0 dejaría
sin comprobar justo el límite exacto, que es para lo que existe.

### Lo que esto cambia, y lo que no

✅ **En `stark-experiment` el modo depuración protege entero** —199 tests con
comprobación de grados—, y ese modo es el que dio *«expected 41 assertions,
received 42»* cuando se añadió el límite regulatorio al circuito.

❌ **En `zk-ssl` sigue sin proteger**: 65 de 172 fallan, y **esos 65 no
comprueban nada** mientras el modo no se arregle.

### ✅ La causa, confirmada con el mensaje

```
transition constraint degrees didn't match
expected: [... 2046, 2046, ... 1023, ...]
actual:   [... 1023, 1023, ...    0, ...]
```

**2046 = 2 × 1023**: una restricción declarada de **grado 2** evaluando como
**grado 1**. Eso ocurre cuando **uno de los dos factores del producto es
constante** en esa traza concreta. Y los `1023 → 0` son restricciones enteras
constantes.

Es exactamente la explicación que esta sección daba desde el principio, y
**la revisión que la puso en duda se equivocaba**.

### Por qué la capa y no los circuitos

| | Índices de cuenta | Bits del camino |
|---|---|---|
| Tests de circuitos | Sintéticos, `is_right = level % 3 == 0` | **Varían** |
| Tests de la capa | 0, 1, 2… asignados en orden | **Casi todo ceros** |

La capa abre cuentas secuencialmente desde el índice 0, y un camino de Merkle
hacia la posición 0 tiene **todos los bits a cero**. La restricción booleana
sobre ese bit es constante, y su grado real cae.

### Qué restricciones exactamente

Cruzando los índices del mensaje con el mapa de `circuit_send`:

| Índices | Grupo | Árbol |
|---|---|---|
| 32–43 | `C_PLACE_A`, `C_PLACE_B`, `C_SIBLING` | Cuentas |
| 44 | `C_BIT_BOOL` | Cuentas |
| 106–113 | `C_FROZEN_ENTRY`, `C_FROZEN_PLACE` | Congelados |
| 114 | `C_FBIT_BOOL` | Congelados |
| 148–160 | `C_PEND_PLACE`, `C_PEND_SIBLING`, `C_PBIT_BOOL` | Pendientes |

**Los tres árboles, y siempre las restricciones que dependen del bit del
camino.** Ninguna otra familia aparece: ni los hashes, ni el saldo, ni el
suministro, ni el límite.

Es la confirmación más estrecha posible de la causa: **los tres caminos
apuntan a la posición 0**, así que sus bits son constantes.

### El arreglo, y su coste

Haría falta que los tests operaran sobre cuentas en índices con **bits
variados** —por ejemplo `0b10101`— en vez de 0 y 1.

⚠️ **No es un cambio pequeño**: abrir cuentas de relleno hasta llegar ahí
cambia `account_count()`, los índices que los tests declaran, y las cifras de
suministro de varios. Son 172 tests. **No está hecho.**

⚠️ **Pero hay una parte barata.** De los tres árboles, el de **pendientes** no
depende de índices que los tests declaren: `allocate_pending` devuelve la
primera posición libre, que es 0. Hacer que empiece en una posición con bits
variados arreglaría `C_PEND_*` **sin tocar ningún test**, y dejaría el
problema reducido a los otros dos árboles.

**No se ha intentado**, y conviene medir cuántos de los 65 caen solo por esa
familia antes de decidir si compensa.

⚠️ **Y el coste de no hacerlo está medido**: 65 tests de la capa **no
comprueban nada** en modo depuración, que es el único que valida los grados
de restricción.

### Versión anterior de esta sección, conservada

> **⚠️ CORRECCIÓN.** Esta sección decía **56** y describía una causa que no
> los explicaba. Medido de nuevo: **son 2**, deterministas, y la causa real
> es distinta de la que se había supuesto. Lo que sigue es la versión
> corregida; lo anterior se conserva más abajo por lo que enseña.

### Los dos, medidos

```
circuit_audit::tests::no_constraint_is_vacuous
circuit_mint_pending::tests::minting_exactly_up_to_the_cap_is_allowed
```

Tres ejecuciones seguidas dan **2 fallos exactos**. No hay intermitencia
—se creyó verla al comparar salidas de compilaciones incrementales
distintas, que es el mismo error de suponer sin medir que esta auditoría
documenta en otros sitios—.

### La causa del segundo, confirmada

`circuit_mint_pending` descompone **`tope − suministro_nuevo` en 63 bits**.
El test alcanza el tope **exactamente**, así que esa resta vale **cero** y
**los 63 bits son cero**.

Las restricciones booleanas sobre esos bits —`bit × (bit − 1)`— tienen grado
real **0** en esa traza concreta, y la comprobación de depuración de
winterfell exige que el grado declarado se realice.

⚠️ **Es un fallo del test positivo, no del negativo.** El negativo espera que
la prueba falle, y falla —por otra razón—, así que **pasa por casualidad**.
El positivo es el que lo delata. Es exactamente la razón por la que §14 exige
escribir los pares.

### Lo que esto cambia de la conclusión anterior

La versión previa decía: *«la suite en modo depuración no protege: los 56
tests que fallan no comprueban nada»*.

✅ **Con 2 fallos, el modo depuración SÍ protege casi entero** —198 de 200—,
y ese modo es el que caza restricciones rotas: es el que dio *«expected 41
assertions, received 42»* cuando se añadió el límite regulatorio al circuito.

⚠️ **Y eso lo vuelve más valioso de lo que se creía**, no menos: 198 tests
comprobando grados de restricción es una red que la documentación daba por
inútil.

### La cifra estuvo rancia y ninguna herramienta lo vio

`check_figures.py` comprueba las cifras de tests que **pasan**. Nada
comprobaba una cifra sobre tests que **fallan**, y por eso 56 sobrevivió a un
factor de 28 de error.

> **Una herramienta que verifica las afirmaciones sobre el éxito no dice nada
> de las afirmaciones sobre el fallo.**

### Versión anterior de esta sección, conservada

**Y no porque el código esté mal.**

Merece estar aquí porque es lo primero
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

⚠️ **Y significa que la suite en modo depuración no protege**: los 65 tests
que fallan no comprueban nada mientras el modo no se corrija.

> ⚠️ **Esta cifra decía 56 y contradecía al título de esta misma sección.**
> Medida el 30 de julio de 2026 con `cargo test -p zk-ssl` sin `--release`:
> **109 pasan, 65 fallan** de 174. Los tres preprints publicaban 56. Cerrarlo
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

### ✅ Resuelto después, y por otro camino

Lo que faltaba no era migrar más llamadas: era **dar al cliente un sustituto
de `prove_transfer`**. Con `send_materials` / `prove_send` y
`claim_materials` / `prove_claim` (§33), la vía antigua se quedó sin nada que
la necesitara y **se retiró entera**:

| Retirado | Tamaño |
|---|---|
| `transfer.rs` completo | 260 líneas |
| `TransferMaterials` y su `impl` | 1.776 B |
| `transfer_materials`, `prove_transfer`, `compute_nullifier` | ~3.000 B |

**Y con ella el límite del cumpleaños.** Hoy todas las llamadas a `commit`
pasan `None` como nullificador, y las únicas escrituras que quedan
—`persistence` y `snapshot`— **restauran de disco lo escrito antes de la
retirada**. Nada genera nullificadores.

~~⚠️ **El árbol se conserva por compatibilidad de formato**: quitarlo cambiaría
el fichero en disco y el instantáneo, y obligaría a migrar ledgers
existentes. Es peso muerto, y está marcado como tal en `accounts.rs`.~~

✅ **Retirado después, con migración verificada.** Ver §36: el formato en
disco y el de instantánea cambiaron, y los datos legados se verifican
contra sus raíces guardadas antes de eliminarse — nunca en silencio.

⚠️ **Y el límite no se ha resuelto: se ha evitado.** El encadenamiento de
raíces que sustituye al nullificador **exige un orden total**, que un solo
nodo da y un sistema distribuido no. Quien distribuya esto lo recupera entero.

### Lo que enseña el rodeo

El plan —*«migrar 35 llamadas y retirar la función»*— estaba **mal formulado
antes de estar bien ejecutado**. Contaba llamadas a una función cuando lo que
sujetaba la vía antigua era **una API pública que no la llamaba**.

Las 35 migraciones no fueron en balde: produjeron cuatro hallazgos —§25, §26,
§29, §30—. Pero el cierre llegó por escribir lo que faltaba, no por terminar
de contar.

> **Un plan expresado en unidades de trabajo puede completarse sin alcanzar
> su objetivo.** Lo que había que preguntar no era «¿cuántas llamadas
> quedan?» sino «¿qué impide borrar esto?».

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

### ✅ Cerrada

| Pieza | Estado |
|---|---|
| `send_materials` / `prove_send` | ✅ **Funciones libres, no tocan la capa** |
| `claim_materials` / `prove_claim` | ✅ **Igual, y sin precedente que copiar** |

Lo demuestra `a_send_without_giving_the_key_to_the_layer`: la capa entrega
materiales sin ver la clave, el cliente prueba en local, la capa verifica y
aplica.

⚠️ **Y el tipo cierra además la fuga hacia la contraparte.**
`TransferMaterials` lleva `receiver: AccountView` —el saldo del receptor—
porque la vía de un paso actualiza las dos hojas. `SendMaterials` lleva
`receiver_id: Digest` y nada más: **no hay campo por donde el saldo pudiera
entrar**.

⚠️ **La tercera fila no tiene precedente.** El cobro es una operación del
receptor que la vía de un paso no tenía, así que su lado cliente hay que
**diseñarlo, no traducirlo**.

`a_whole_payment_without_giving_any_key_to_the_layer` lo demuestra entero:
**ni la clave del pagador ni la del receptor llegan a la capa.** Lo único que
la capa aporta son caminos y raíces; lo único que recibe son pruebas que
verifica.

⚠️ **Y una asimetría que explica la pieza de ISO que falta.** `send_materials`
recibe el identificador del receptor; `claim_materials` recibe el **aviso
completo**, porque **la capa no sabe qué pendiente es de quién** —esa es la
privacidad del diseño— y por tanto no puede entregarlo.

> **Que ISO 20022 no transporte el aviso no es un olvido del estándar: es una
> consecuencia de que el sistema no sepa de quién es cada pendiente.**

### Cuatro rondas de compilación para una función

| Error | Causa |
|---|---|
| `salt_de` invisible desde `metrics.rs` | Ayudante privado de `mod tests` |
| `SendReceipt` invisible desde `client.rs` | Módulo público, tipos no reexportados |
| `AccountRecord` ≠ `AccountView` | **`account_view` ya hacía la conversión, al lado** |
| Campo `commitment` quitado | Se leyó en un `grep` y **se atribuyó a la estructura equivocada** |

Los cuatro son de la misma familia: **suponer en vez de mirar**. El tercero
repite exactamente el fallo de §27 —el mecanismo correcto estaba en el mismo
fichero y no se copió— y el cuarto es peor: **un `grep` sin contexto atribuyó
un campo a la estructura de al lado.**

---

## 34. ⚠️ Un mensaje ISO malformado movía dinero

**Cuarto hallazgo del ejercicio de contraste, en el cuarto intento.**

`MsgId` y `EndToEndId` son `Max35Text` **obligatorios** en ISO 20022. La
validación de producción comprobaba divisa, importe cero e IBAN, y **no los
identificadores**.

| Mensaje | Antes | Ahora |
|---|---|---|
| `MsgId` vacío | **Se liquidaba** | `FF10`, sin tocar el ledger |
| `MsgId` de 36 caracteres | **Se liquidaba** | `FF10` |
| `EndToEndId` vacío o largo | **Se liquidaba** | `FF10` |

⚠️ **Y la consecuencia no era solo formal**: el `pacs.002` de respuesta salía
con `original_msg_id` vacío, así que **el emisor no podía correlacionar la
respuesta con su petición**. Dinero movido y acuse sin referencia.

### Cómo apareció

Contrastando los tres tests del crate `iso-bridge` —superado, **no usado por
producción**— contra la vía real. Uno de ellos era
`empty_message_id_is_rejected_before_touching_zk_core`.

**El crate superado comprobaba algo que la producción no.** Es exactamente el
patrón de §24 con `pending.rs` y de §25 con `settlement-layer`.

| Intento | Módulo contrastado | Hallazgo |
|---|---|---|
| 1 | `pending.rs` | Una propiedad de seguridad probada solo sobre el modelo |
| 2 | `settlement-layer` | La regresión del límite regulatorio |
| 3 | `two_phase` vs las demás | Ninguna operación dejaba rastro en el registro |
| 4 | `iso-bridge` | **Un mensaje malformado movía dinero** |
| 5 | `zk-core` | Una restricción correcta **pero sin test** |

> **Cinco de cinco.** Ningún otro método de esta auditoría —herramientas de
> mutación, comprobadores de columnas, revisión de circuitos— ha tenido ese
> rendimiento.

### El quinto es de otra clase, y conviene distinguirlo

`zk-core` tenía `wrong_balance_not_matching_leaf_fails_constraints`.
`circuit_send` **no** tenía equivalente: nadie había comprobado que declarar
más saldo del que hay en la hoja rompa la prueba.

⚠️ **La restricción existía y funcionaba.** El camino de Merkle no reconstruye
`root_old` si el saldo declarado no es el de la hoja, y el test nuevo
—`a_lie_about_the_current_balance_is_rejected`— lo confirma a la primera.

**Lo que faltaba era la demostración**, y faltaba justo donde importa: la
capa lo cazaba con `StaleState`, que es una comprobación **evitable por quien
construya su propia traza**. Es el mismo hueco que §25, sin la regresión.

> **Los cuatro primeros encontraron fallos. El quinto encontró una
> afirmación sin respaldo**, que en un sistema que se publica con DOI es un
> hallazgo distinto pero no menor.

### ⚠️ Dónde deja de funcionar el método

Aplicado a `halo2-experiment` (27 tests), `plonk-experiment` (36) y
`nova-experiment` (3), **el contraste no produce nada**, y el motivo importa
más que el resultado.

Esos crates **no son implementaciones superadas de lo mismo**: son el estudio
comparativo. Sus tests negativos prueban **los primitivos de cada sistema**
—que un testigo inválido rompe un circuito Halo2, que el gadget Poseidon
detecta un hash falso— no las propiedades de liquidación.

`wrong_hash_fails_circuit` comprueba Poseidon en Halo2. Producción usa Rescue
en Winterfell. **No es la misma propiedad expresada de otra forma: es otra
propiedad.**

> **El contraste rinde cuando el módulo es una implementación superada de lo
> mismo, no cuando es simplemente otro módulo.** Los cinco hallazgos vienen
> de comparar dos formas de hacer lo mismo; comparar dos cosas distintas no
> dice nada.

Queda declarado para que nadie repita el ejercicio sobre estos tres
esperando el mismo rendimiento. `settlement-prover` y `ceremony` no tienen
tests propios.

### Comparación con las otras cuatro del contraste

También quedó comprobado que **`derive_custodian_id` y `derive_public_id` no
coinciden** —si lo hicieran, un titular de cuenta podría hacerse pasar por
custodio— y esa sí estaba fijada con un test desde antes.

### El código de rechazo, anotado y no resuelto

Se usa `FF10`, que **ya estaba verificado en este código**. `FF01` (Invalid
File Format) podría ser más preciso para un elemento que incumple el esquema.

⚠️ **No se ha inventado uno**, que es justo el error que §21 corrigió tres
veces —`TECH`, `AG01`, `AM12`—. Anotado para quien tenga el catálogo.

### Lo que el test fija

`a_malformed_message_is_rejected_before_moving_money` prueba los cuatro casos
y termina comprobando **el saldo**:

> El código de rechazo puede discutirse; **que el saldo no se mueva, no**.

Y su par positivo, `identifiers_of_exactly_35_characters_are_accepted`,
porque **35 es el máximo permitido, no el primero prohibido**.

---

## 35. El experimento de índices altos: ejecutado, y con una trampa antes

La hipótesis de §20 —los grados caen porque los bits de los caminos de
Merkle son constantes, y lo son porque las cuentas están en los índices 0
y 1— tenía un experimento diseñado para decidirla:
`experimento_indices_altos`, con el criterio escrito **antes** de ver el
resultado.

### ⚠️ La primera ejecución no ejecutó nada

El test estaba **anidado dentro de**
`a_restart_does_not_renew_an_exhausted_custodian_quota`: compilaba, no se
registraba, y no corría. Es la primera forma del catálogo de §17
(`#[test]` dentro de una función), apareciendo por segunda vez en el
proyecto.

Lo insidioso no es el anidamiento sino **cómo se habría leído**:

| Comando | Salida con el test anidado | Lectura ingenua |
|---|---|---|
| Release, filtrando `test result` | `ok. 0 passed; 173 filtered out` | «ok» |
| Depuración, filtrando `panicked at` | *(vacío)* | «no hay panic → pasa» |

Y la tabla de decisión pre-registrada decía: *«Pasa → el relleno basta»*.
Un experimento que no corrió habría confirmado la hipótesis en su rama
más optimista. **Pre-registrar el criterio protege contra reinterpretar
el resultado; no protege contra leer la ausencia de resultado como
resultado.** La tabla tenía tres filas y necesitaba cuatro:

| Si en depuración… | Entonces |
|---|---|
| Pasa | El relleno basta |
| Falla solo en `C_PEND_*` | Arregla dos de tres |
| Falla igual | La hipótesis es falsa |
| **`0 passed` o filtro vacío** | **No corrió: no concluir nada** |

✅ **`tools/check_tests.py` lo detectó** —línea y diagnóstico exactos—
en su primer uso real desde que se validó en los dos sentidos. Es la
primera de las herramientas de auditoría que paga su construcción. La
lección operativa: **ejecutarla es paso 0 de cualquier tanda
experimental**, no una comprobación posterior.

### El resultado, con el test ya a nivel de módulo

Release: pasa (el escenario es válido). Depuración: panic de grados en la
generación de la prueba de cobro, con **21 restricciones** desviadas de
162 —todas contiguas, ninguna fuera de la familia del árbol de
pendientes—:

| Restricciones | Declarado → real | Causa |
|---|---|---|
| `C_PEND_ENTRY_A`, `C_PEND_ENTRY_B`, `C_PEND_PLACE`, `C_PEND_SIBLING` (20) | 2046 → 1023 | El factor `pbit` es constante: un grado menos |
| `C_PBIT_BOOL` (1) | 1023 → **0** | `bit × (bit − 1)` con bit constante: idénticamente cero |

Y lo que **dejó de aparecer** confirma la otra mitad: las familias de
cuentas (`C_PLACE_*`, `C_SIBLING`, `C_BIT_BOOL`) y de congelados
(`C_FROZEN_*`, `C_FBIT_BOOL`), que fallaban antes del relleno, ahora
coinciden.

**Veredicto: rama 2 de la tabla.** El relleno arregla dos de tres
árboles. La posición del pendiente sale de `allocate_pending()` —un
contador propio que arranca en 0, independiente de los índices de
cuenta— y ningún relleno de cuentas la toca.

### Tres decisiones que el resultado impone

**1. El refactor de los 20 tests no se hace.** Su objetivo era que la
suite pasara en depuración. Con el pendiente sin resolver, cualquier test
con transferencia de dos fases seguiría fallando: el trabajo no compra el
objetivo. El experimento existía para decidir si merecía la pena tocar
los tests antes de tocarlos; la respuesta es no, tal como está.

**2. La vía barata para el pendiente se rechaza, y queda escrito por
qué.** Avanzar el contador en los tests para que el pendiente caiga en
una posición con bits variados haría pasar la validación **fabricando
testigos no representativos**: el primer pago real de cualquier
despliegue también cae en la posición 0, y su traza degrada igual. La
degradación no es un artefacto del escenario de test —es una propiedad
de toda traza con posición baja—. Ocultarla en los tests sería declarar
una propiedad mientras se esconde su contraria.

**3. Lo que queda es trabajo de circuito, y va a Aplazado.** O bien se
acepta y documenta que la validación en depuración exige testigos con
bits variados (el statu quo, ahora con causa exacta y nombres de
restricción), o bien se reformulan las restricciones para que el grado
declarado se realice en toda traza. Lo segundo es un cambio de circuito
con su propio experimento, no un parche de tests.

### El experimento se borra

Su comentario lo decía: existe para contestar una pregunta y se borra al
contestarla. La receta de reproducción queda aquí: cuentas de relleno
hasta que el emisor caiga en el índice 21 (`0b10101`) y el receptor en
22, un pago de dos fases, y ejecutar sin `--release`. El diff de grados
esperado es el de la tabla de arriba.

## 36. El árbol muerto de nullificadores: retirado con migración verificada

§32 dejó el árbol de nullificadores como **peso muerto**: nada lo escribía
—todas las llamadas a `commit` pasaban `None`— y se conservaba solo porque
quitarlo cambiaba el formato en disco y el de instantánea. Esta sección
documenta su retirada y las decisiones que la gobiernan.

### Lo que se eliminó

| Pieza | Detalle |
|---|---|
| `SovereignLayer.nullifiers` | El campo y sus tres inicializaciones |
| `nullifier_root()` | Y su comentario de peso muerto en `accounts.rs` |
| Parámetro `nullifier` de `commit()` | Muerto en las diez llamadas; la firma queda en dos parámetros |
| `root:nullifier` y `null:{pos}` | Ya no se escriben al disco |
| Cabecera y sección de nullificadores de la instantánea | El formato pasa de `ZKSSL3` a `ZKSSL4` |

Y de paso, tres capas de documentación rancia de la vía de un paso que
seguían fusionadas sobre la API viva: el ejemplo de `lib.rs` que llamaba a
`transfer_materials`/`prove_transfer` —funciones que ya no existen—, el
protocolo de cinco pasos de `client.rs`, y una doc huérfana de
`compute_nullifier` que había quedado pegada encima de `SendMaterials`.

### La decisión: migración verificada, nunca silenciosa

Quitar el árbol dejaba tres opciones para los ledgers e instantáneas
existentes. La ruptura dura incumplía la promesa del formato —*«una copia
de archivo debe poder leerse dentro de diez años»*—. La migración
silenciosa —ignorar y borrar— violaba el principio de `persistence`:
descartar datos sin verificarlos **es** cargar en silencio.

Lo implementado es la tercera:

**Ledger (`sled`).** Al abrir, si existen claves `null:{pos}`, se
reconstruye el árbol legado, **se comprueba contra la `root:nullifier`
guardada**, y solo si coincide se eliminan —claves y raíz— en un lote
atómico. Si no coincide, `IntegrityFailure`, igual que un árbol vivo.
Claves sin raíz que las respalde también detienen el arranque: no hay
contra qué verificarlas.

**Instantáneas.** Se exporta solo `ZKSSL4`. Se importa `ZKSSL4` **y
también `ZKSSL3`**: los nullificadores de una copia v3 se reconstruyen en
un árbol temporal, se verifican contra la raíz que la copia declara, y se
descartan. Una v3 con la raíz manipulada se rechaza.

Lo protegen dos tests nuevos —`a_v3_snapshot_with_nullifiers_imports_verified`
y `a_v3_snapshot_with_a_forged_nullifier_root_is_rejected`— que fabrican
los bytes v3 a mano, porque el código actual ya no puede exportarlos: es
la única forma de probar que una copia antigua sigue leyendo.

### Lo que encontró la retirada: la geometría codificada

`a_tampered_snapshot_is_rejected` falló tras el cambio de formato:
codificaba la cabecera v3 en una constante —cinco raíces, cuatro
contadores, 264 bytes— y su byte manipulado dejó de caer en el registro
de cuenta que dice alterar para caer en el registro de transiciones, que
se rechaza como `Malformed` y no como el `IntegrityFailure` exacto que el
test exige. **El barrido previo al parche no podía encontrarlo**: buscó
«nullifier» por nombre, y la constante no lo nombra — codifica su
tamaño.

Ya le había pasado al mismo test en la dirección contraria: su primera
versión contaba desde el final del fichero, y añadir el registro
convirtió aquel byte en historial. Creció el formato y se rompió;
encogió y se volvió a romper.

> **Un cambio de formato hay que contrastarlo con todo test que codifique
> desplazamientos, no solo con los que nombran lo cambiado.**

El riesgo declarado por adelantado —que `verify_chain()` rechazara el
registro vacío de los tests v3— **no se materializó**: los dos tests
nuevos pasaron a la primera. Lo que falló fue lo no previsto, que es
exactamente para lo que sirve declarar lo previsto.

### Lo que NO cambia

- **El límite del cumpleaños sigue evitado, no resuelto** (§13, §32).
  Quien distribuya esto recupera el problema entero, y entonces un árbol
  de nullificadores —u otro mecanismo de gasto único— vuelve a hacer
  falta. Esta retirada elimina peso muerto de un nodo único; no toma
  ninguna decisión sobre el diseño distribuido.
- Los circuitos `circuit_settlement` y `nullifier_tree` de
  `stark-experiment` **se conservan**: documentan la vía comparada en los
  papers y no pesan sobre la capa.
- `settlement-layer` —el crate Groth16 autónomo— mantiene su propio
  árbol: es el artefacto histórico de la comparativa, no la capa.

## 37. El grado que depende del testigo

§35 dejó aplazada la reformulación de los grados del árbol de pendientes.
Al abrirla se ve que estaba mal planteada en dos sentidos, y esta sección
corrige el planteamiento antes de tocar nada.

### 37.1 Una sospecha, cerrada leyendo

`circuit_claim.rs` usa el `pbit` de la fila **siguiente** para las
restricciones del pendiente (línea 848, `next[COL_PBIT]`) e impone su
booleanidad sobre la fila **actual** (línea 899, `current[COL_PBIT]`). Si
algún valor se usara sin comprobarse, un probador podría colocar ahí algo
que no fuera 0 ni 1.

**No hay hueco.** El segmento que llena `COL_PBIT` ocupa las filas
`61 × 8 = 488` a `(61 + 31) × 8 + 7 = 743`, y la traza tiene 1024 filas.
Todo `pbit` usado como `next` en la transición *r* se comprueba como
`current` en la transición *r+1*, que existe siempre porque el segmento
termina 280 filas antes del final.

Se registra la sospecha resuelta en negativo, no solo las que dan fruto:
un documento que solo anota los aciertos no dice cuánto se ha mirado.

### 37.2 El diagnóstico, y por qué «declarar menos» no es una salida

Winterfell no exige que el grado real sea **menor o igual** que el
declarado: exige que **coincida**. El mensaje del test de `range_check` lo
dice literal —*«expected [63,0,63], actual [0,0,0]»*—. Bajar la
declaración rompería el caso no degenerado, que es el normal.

Queda una sola dirección: **que el grado real no dependa del valor del
testigo**. Y ahí los casos no son uno, son dos, con pronósticos opuestos.

**Caso A — el árbol de pendientes.** `COL_PBIT` guarda los bits de la
posición. Con posición 0 son todos cero, la columna es idénticamente nula,
`pbit × (pbit − 1)` es el polinomio cero, y en `C_PEND_*` los términos
multiplicados por `pbit` desaparecen. §35 descartó adelantar el contador
**en los tests** con buen criterio: el primer pago real también cae en la
posición 0. Pero de ahí no se sigue que haya que tocar el circuito. Si la
capa **nunca asigna la posición 0** (`pending.rs:131`, `next: 0`), la
producción deja de ser degenerada, y basta un solo bit no nulo para que la
columna no sea el polinomio cero.

**Caso B — los otros dos circuitos, y probablemente sin arreglo.**
`circuit_mint_pending` degenera cuando el margen del tope es exactamente
cero, y `range_check` cuando la diferencia es cero. Los dos son **valores
legítimos del dominio**: el circuito de cumplimiento necesita
`amount == balance`, verificado en Groth16 y Halo2. No se pueden diseñar
para que no ocurran, y **no se arreglan reformulando**: si una columna es
idénticamente cero, cualquier restricción booleana sobre ella es el
polinomio cero, se escriba como se escriba.

Si el caso B es irreducible, lo correcto no es perseguirlo sino
**declararlo**: la comprobación de grados de winterfell es incompatible con
restricciones cuyo grado depende del valor, eso es una propiedad de la
herramienta, y el precio es el que ya se paga.

### 37.3 Experimento PRE-REGISTRADO: desplazar la asignación

**Se escribe antes de ejecutarlo.** La tabla de decisión de abajo se fija
ahora para que el resultado no pueda reinterpretarse después.

**Hipótesis.** Si `PendingTree::new` empieza en `next: 1` en vez de `next:
0`, las 21 restricciones desviadas del caso A dejan de desviarse, porque
`COL_PBIT` pasa a tener al menos un bit no nulo.

**Procedimiento.** Cambiar `next: 0` por `next: 1` en `pending.rs:131` y
ejecutar `cargo test -p zk-ssl` **sin** `--release`.

| Resultado | Lectura | Decisión |
|---|---|---|
| 0 fallan | La hipótesis se sostiene y el caso A se cierra | Evaluar el coste del cambio: una hoja de 2³² perdida y la migración de registros con un pendiente en la 0 |
| Fallan menos de 65, ninguno `C_PEND_*` ni `C_PBIT_BOOL` | El caso A se cierra; lo que queda es caso B u otra causa | Cerrar A, abrir entrada nueva para el resto |
| Siguen fallando `C_PEND_*` o `C_PBIT_BOOL` | La hipótesis **no** se sostiene: un bit no nulo no basta | Descartar el desplazamiento y volver al planteamiento |
| Salida vacía, `0 passed`, o error de compilación | **El experimento no corrió** | No concluir nada |

La cuarta fila es la lección de §35, y va de serie desde entonces.

⚠️ **Lo que este experimento no decide.** Aunque salga la primera fila, el
cambio no es gratis ni obviamente correcto: desplazar la asignación
significa que un registro existente con un pendiente en la posición 0
queda en un estado que el código nuevo no vuelve a generar. Eso es una
migración, y se decide como se decidió la del §36 — verificando, no
descartando en silencio.

### 37.4 Resultado: tercera fila, el desplazamiento se descarta

Ejecutado el 30 de julio de 2026 con `next: 1` en `pending.rs:131`:
**108 pasan, 66 fallan** de 174, frente a los 65 de partida.

⚠️ **La tabla que sigue estaba mal atribuida y se corrige en §37.6.** El
vector se tomó del **primer** panic del fichero, que resultó ser de un
circuito de 512 filas y 76 restricciones —ninguno de los dos que tocan el
árbol de pendientes, que son de 1024—. Se conserva tachada porque así se
escribió:

~~El vector de grados de una traza que falla, comparado entero:~~

| Restricciones | Declarado | Real | |
|---|---|---|---|
| Índices 32–43 (12) | 1022 | **511** | siguen desviadas |
| Índices 44–51 (8) | 1022 | 1022 | **recuperadas** |
| Índice 52, `C_PBIT_BOOL` | 511 | **0** | sigue desviada |

~~**13 desviadas donde antes había 21.**~~ **No.** Ese 13 es de otro
circuito, y no cambió con el experimento. Y aun así, tercera fila de la tabla
de §37.3 —siguen fallando `C_PEND_*` y `C_PBIT_BOOL`—, así que **el
desplazamiento se descarta** y el cambio se revierte. La tabla se escribió
antes precisamente para que una mejora parcial no se leyera como éxito.

Y no es siquiera neutral: **los fallos suben de 65 a 66**. Un perfil de
grados distinto hace fallar a un test que antes no fallaba, así que el
cambio no se sostiene ni como paso intermedio.

### 37.5 ~~Lo que el experimento sí establece~~ — retirada entera

⚠️ **Esta subsección se retira: no establecía nada.** Afirmaba que ocho
restricciones habían recuperado su grado y que el mecanismo quedaba
confirmado. La medición correcta (§37.6) demuestra que **no cambió
absolutamente ningún grado**. Se conserva el texto porque así se publicó, y
porque un documento que borra sus errores no permite contarlos.

### 37.5-bis Texto retirado

**El mecanismo queda confirmado.** Ocho restricciones pasaron a realizar su
grado en cuanto la columna dejó de ser idénticamente nula. La causa es la
que §35 identificó.

**Y queda claro por qué un bit no basta.** La posición 1 tiene **31 de sus
32 bits a cero**. Solo el nivel 0 aporta un valor no nulo, y los treinta y
un niveles restantes siguen con su `pbit` constante. No hacía falta que la
columna fuera nula: basta con que lo sea *por tramos*.

De donde se sigue lo que ninguna asignación secuencial arregla: los
primeros millares de posiciones tienen casi todos los bits altos a cero,
sea cual sea el punto de partida. **El caso A no se cierra desplazando el
contador.**

⚠️ **Una hipótesis nueva, sin verificar y sin decidir.** Una asignación
que **permutara** el contador de forma biyectiva —inversión de bits, por
ejemplo— daría posiciones con bits variados desde el primer pago, y al ser
biyectiva **no reintroduciría colisiones**, que es lo que hundió al árbol de
nullificadores (§13, §36). Se anota como candidata a un experimento futuro,
no como conclusión: no se ha medido, y una posición derivada de cualquier
función del contador merece mirarse con la desconfianza que ya costó una
vez. Derivar la posición de un **hash** sí reintroduciría el límite del
cumpleaños entero, y esa vía queda descartada de antemano.

## 37.6 El experimento fue NULO: modificó el modelo, no lo que corre

La comparación correcta —agrupando los panics **por circuito**, que es lo
que §37.4 no hizo— da un resultado que ninguna fila de la tabla prevé.

| Circuito (restricciones) | Panics | Desviadas, `next: 0` | Desviadas, `next: 1` |
|---|---|---|---|
| 57 | 3 | 5 | **5** |
| 76 | 13 | 13 | **13** |
| 114 | 5 | 22 | **22** |
| 125 | 1 | 27 | **27** |
| 125 | 2 | 21 | **21** |
| 162 (`circuit_claim`) | 39 | 43 | **43** |

**Idénticos.** Mismos circuitos, mismos 63 panics, mismos índices. El
cambio no alteró un solo grado.

### Por qué

`pending.rs` define `PendingTransfers`, con su campo `next` y sus ocho
tests. **La capa no lo usa.** El estado real vive en
`SovereignLayer.next_pending` (`lib.rs:544`) y las posiciones las reparte
`two_phase::allocate_pending`; de `pending.rs` la producción solo importa
la función libre `pending_commitment`.

El experimento cambió `pending.rs:131`. Es decir: **modificó un modelo que
no se ejecuta**, y por eso los grados no se movieron. El fallo número 66 no
era un panic de grados sino la aserción `assert_eq!(pos, 0)` de uno de los
ocho tests del propio modelo.

### La clase de error, que este proyecto ya tenía documentada

§8.2 del preprint comparativo dice, con estas palabras, que *un módulo
prototipo lleva ocho tests demostrando las propiedades del diseño en dos
fases, y que producción usa una función de ese módulo y ninguna de sus
estructuras de datos*. Es el mismo módulo. Es el mismo error. Y esta vez
sobrevivió a un protocolo pre-registrado.

> **El pre-registro fija los criterios; no comprueba que la intervención
> llegue a lo que se ejecuta.** Un experimento puede estar impecablemente
> diseñado, correr entero, dar una salida limpia y no haber tocado el
> sistema.

La tabla de decisión necesita una **quinta fila**, hermana de la cuarta que
§35 añadió:

| Resultado | Lectura | Decisión |
|---|---|---|
| El perfil medido **no cambia en absoluto** | La intervención no llegó a la traza | **Experimento nulo**: no es rechazo, es que no se midió nada |

Y un paso previo obligatorio, antes de leer ningún resultado: **demostrar
que la intervención alcanza lo que se ejecuta**, no suponerlo por el nombre
del fichero.

### Qué queda en pie

Solo esto: la hipótesis de §37.3 **sigue sin comprobar**. Lo que se sabe es
que las desviaciones alcanzan **cinco circuitos**, no uno, y que en
`circuit_claim` caen en tres bloques —índices 32–44, 106–114 y 140–160— que
por el mapa de constantes corresponden a las subidas al árbol de
**cuentas**, al de **congelados** y al de **pendientes**. Los tres son bits
de camino constantes. El diagnóstico de §35 se amplía: no es el pendiente,
son todos los caminos.

### El experimento, reformulado

**Hipótesis** (sin cambios): si las posiciones del árbol de pendientes
dejan de tener los bits de camino constantes, las restricciones `C_PEND_*`
y `C_PBIT_BOOL` realizan su grado declarado.

**Procedimiento corregido.** Sobre `two_phase::allocate_pending`, que es lo
que reparte de verdad: iniciar la búsqueda en 255 y devolver como mínimo
255. Se elige 255 —`0b11111111`— porque pone **ocho bits a uno**, no uno
solo, y porque el bucle de búsqueda recorre 255 posiciones y no dos mil
millones.

**Comprobación de alcance, antes de leer nada.** El perfil de grados del
circuito de 162 restricciones **tiene que cambiar**. Si sale idéntico, la
intervención sigue sin llegar y no se concluye nada.

## 37.7 El experimento corregido: la hipótesis se sostiene

Intervención sobre `two_phase::allocate_pending` —búsqueda desde 255 y
posición mínima 255, ocho bits de camino a uno—, ejecutado el 30 de julio
de 2026.

**Comprobación de alcance, primero:** el perfil de grados **cambia**. La
intervención llega a las trazas, a diferencia de la de §37.4.

**Resultado: 110 pasan, 64 fallan**, frente a 65 de referencia. Y el perfil,
circuito a circuito:

| Circuito | Desviadas antes | Desviadas ahora | Bloque de pendientes |
|---|---|---|---|
| 57 | 5 | 5 | — |
| 76 | 13 | 13 | — |
| 114 | 22 | 22 | — |
| 125 | 21 y 27 | **6** | **desaparece** (86–106) |
| 160 | — | 22 | — |
| 162 | 43 | **22** | **desaparece** (140–160) |

En el circuito de 162 se van exactamente las veintiuna que §35 había
identificado por nombre —veinte `C_PEND_*` más `C_PBIT_BOOL`—, y en el de
125 desaparece un bloque de veintiuna con la misma firma. **En ninguno de
los seis perfiles queda una sola desviación en el rango de pendientes.**

**Segunda fila de la tabla de §37.3**: menos de 65 fallos, ninguna
`C_PEND_*` ni `C_PBIT_BOOL`. La hipótesis se sostiene: **si los bits de
camino dejan de ser constantes, las restricciones realizan su grado.**

### Lo que esto cierra y lo que no

**Cierra el diagnóstico**, que era lo que §35 dejó abierto y §37.4 no llegó a
tocar. **No implementa nada**: la intervención es una medición y se
revierte. Convertirla en diseño exige decidir si se paga el precio de que
las posiciones del árbol de pendientes dejen de ser secuenciales —hojas
desperdiciadas, y la pregunta de siempre sobre registros ya escritos—.

Y sobre todo, **no basta**. Quedan 64 fallos, concentrados en dos bloques
que aparecen en casi todos los circuitos: los índices bajos (32–44) y los
medios (105–114). Por su posición corresponden a las subidas al árbol de
**cuentas** y al de **congelados**, que sufren lo mismo por la misma razón:
las cuentas viven en los índices 0 y 1, y el árbol de congelados suele
estar vacío.

⚠️ Esa atribución por posición es **plausible y no verificada**. Hoy ya
se ha atribuido un vector al circuito equivocado una vez (§37.4); antes de
afirmarlo hay que mapear índices contra las constantes del circuito
concreto, como se hizo con el bloque de pendientes.

El pronóstico de la entrada 6 cambia con esto: **no es «no tiene
arreglo»**, es «tiene arreglo conocido y hay que decidir su precio, en los
tres árboles y no solo en uno».

## 38. ⚠️ Ocho restricciones que se sobrescriben antes de llegar al verificador

**Encontrado el 30 de julio de 2026**, mapeando índices de restricción para
verificar una atribución de §37.7. No se buscaba esto.

### El solapamiento

En `circuit_send` y en `circuit_claim`, idénticamente:

```rust
const C_TRANSPORT: usize = C_SUPPLY + 1;    // 15 (7 + id receptor 4 + aleatorio 4)
const C_ID_CONST:  usize = C_TRANSPORT + 7; // 4
```

El comentario declara que `C_TRANSPORT` ocupa **quince** ranuras, y el
código de evaluación las escribe:

| Escritura | Ranuras | Qué impone |
|---|---|---|
| `result[C_TRANSPORT + k]` | +0 a +6 | Las siete columnas de transporte son constantes |
| `result[C_TRANSPORT + 7 + i]` | +7 a +10 | **La identidad del receptor es constante** |
| `result[C_TRANSPORT + 11 + i]` | +11 a +14 | **El aleatorio es constante** |

Pero el grupo siguiente empieza **siete** ranuras después, no quince:

| Constante | Ranura | Pisa |
|---|---|---|
| `C_ID_CONST` | +7 a +10 | las cuatro del **id del receptor** |
| `C_SBIT_BOOL` | +11 a +12 | dos del **aleatorio** |
| `C_FIRST_S` | +13 a +14 | las otras dos |

Las escrituras posteriores ganan. **Ocho restricciones se calculan y se
descartan**: las que imponen que la identidad del receptor y el aleatorio
del pagador no varíen entre filas.

Nada más las fija: `get_assertions` no ancla `COL_R_ID` ni `COL_SALT` en
ninguna fila, y las únicas otras lecturas de esas columnas están en
`C_PEND_IN`, que construye el compromiso a partir de la fila en curso.

### Lo que NO se afirma

⚠️ **No está establecido que esto sea explotable.** Lo verificado es
estático: las ranuras se escriben dos veces y la segunda gana. Que un
probador malicioso pueda aprovecharlo —por ejemplo poniendo una identidad
en la fila donde se construye el compromiso y otra en el resto— exige un
test discriminante que aún no existe. **Hasta que exista, esto es un
defecto de construcción de dudosa consecuencia, no una rotura demostrada.**

La cautela no es retórica: el mismo día de este hallazgo, tres
afirmaciones sobre grados resultaron mal atribuidas por leer un vector sin
comprobar de qué circuito era (§37.4, §37.6).

### Por qué las herramientas no podían encontrarlo

El detector de restricciones vacuas perturba el testigo y comprueba que
**cada ranura declarada reaccione**. Una ranura sobrescrita **sí
reacciona** —con la restricción que ganó—, así que el detector la ve sana.
No puede notar que ahí vivía otra restricción distinta que se perdió.

> **Una herramienta que verifica que cada ranura hace algo no puede
> detectar que la ranura hace lo que no era.**

Es la misma forma que §8.2 del preprint: una propiedad demostrada sobre
algo que no es lo que corre. Aquí, una restricción escrita sobre algo que
no llega.

### Experimento PRE-REGISTRADO: comprobar que están muertas

Antes de discutir explotabilidad hay que confirmar la premisa.

**Procedimiento.** Comentar las dos escrituras de `result[C_TRANSPORT + 7 +
i]` y `result[C_TRANSPORT + 11 + i]` en `circuit_claim` y en
`circuit_send`, y ejecutar la suite en release.

| Resultado | Lectura | Decisión |
|---|---|---|
| Todo pasa igual (174 y 201) | Las ocho eran código muerto: premisa confirmada | Diseñar el test discriminante de explotabilidad |
| Algo falla | No estaban muertas: el análisis estático se equivoca | Rehacer el mapa de índices |
| No compila, o salida vacía | El experimento no corrió | No concluir nada |
| El perfil no cambia donde debería | La intervención no llegó | Nulo, como §37.4 |

Las dos últimas filas son las lecciones de §35 y §37.6, y van de serie.

### 38.1 Confirmado: estaban muertas

Ejecutado el 30 de julio de 2026, comentando las ocho escrituras en los dos
circuitos: **201 y 174, cero fallos**. Primera fila de la tabla.

Combinado con el análisis estático es concluyente, y no solo indicio: las
ranuras las escriben igualmente `C_ID_CONST`, `C_SBIT_BOOL` y `C_FIRST_S`,
así que el contenido final del vector es **idéntico con y sin ellas**. Las
ocho restricciones no existían para el verificador.

### 38.2 ⚠️ Y siguiendo el hilo, algo peor: SOSPECHA, no hallazgo

Al preguntar qué se pierde exactamente con esas ocho, aparece una pregunta
que no depende del solapamiento y que **no sé responder leyendo**.

En `circuit_claim`, `COL_R_ID` —la identidad del receptor con la que se
reconstruye el compromiso— aparece **cuatro veces en todo el fichero**: la
declaración de la constante, la construcción de la traza por el probador
honesto, la restricción de constancia que resultó estar muerta, y la
lectura de `C_PEND_IN` que arma el compromiso.

Por otro lado, `C_PK_CHECK` impone que la identidad **derivada de la clave
de gasto** coincida con `COL_ACC_ID`, la cuenta que cobra.

**No he encontrado ninguna restricción que ate `COL_R_ID` a `COL_ACC_ID`**,
ni ninguna aserción de frontera que fije `COL_R_ID`.

Si eso es exacto, el circuito demuestra dos cosas verdaderas por separado:

1. quien prueba tiene la clave de la cuenta que cobra, y
2. existe en el árbol un pendiente cuyo compromiso es
   `H(H(COL_R_ID, salt), importe)`,

**sin exigir que la identidad del pendiente sea la de la cuenta que
cobra.** La comprobación de titularidad que el diseño describe —*«reconstruir
el compromiso con su propia identidad»*— la haría la **capa** al construir
la traza, no el circuito.

De ser así, quien conozca los materiales de cobro —posición, aleatorio e
importe— podría cobrar un pendiente ajeno construyendo su propia traza. Y
el pagador **los conoce todos**: él eligió el aleatorio.

### Lo que NO se afirma, y por qué importa decirlo

⚠️ **Esto es una sospecha derivada de lectura estática, no un fallo
demostrado.** No he excluido que exista un vínculo indirecto —por ejemplo
a través de los carriles de entrada del hash, que `C_INPUT` ata a
`COL_ACC_ID` en la primera fila y `C_PEND_IN` ata a `COL_R_ID` en su
propia fase—.

Los tests `nobody_else_can_claim_a_pending_transfer` y
`a_third_party_cannot_claim_the_pending` **pasan**. Eso no zanja nada: van
por la API de la capa, y la pregunta es qué ocurre con una traza
construida a mano. Es exactamente la forma de §8.1 del preprint —un
límite impuesto por la capa y no por el circuito, evitable por quien
construya su propia traza— y la de §8.3 —comparar un valor no es intentar
el ataque—.

### Experimento PRE-REGISTRADO: el test discriminante

**Procedimiento.** En `circuit_claim`, construir una traza de cobro
internamente coherente en todo lo demás, con `COL_R_ID` = identidad de la
víctima y `COL_ACC_ID` = identidad del atacante, que tiene su propia clave.
Generar la prueba y verificarla.

| Resultado | Lectura | Decisión |
|---|---|---|
| La prueba **verifica** | El circuito no ata las dos identidades: **fallo de solidez en la vía de producción** | Parar todo lo demás; corregir el circuito y publicar |
| La prueba **se rechaza** | Existe el vínculo, por una vía que la lectura no vio | Documentar cuál es, y cerrar la sospecha |
| Falla por otra restricción | El testigo no es discriminante (§16.5, ya ocurrió tres veces) | Rehacer el testigo hasta que solo falle lo que se prueba |
| No compila o salida vacía | No corrió | No concluir nada |

## 39. ⚠️ FALLO GRAVE: el cobro no demuestra que el pendiente fuera tuyo

**Confirmado el 30 de julio de 2026.** No hizo falta escribir ningún test:
lo demuestra la suite que ya pasaba.

### El hecho

En `circuit_claim::tests::scenario()`:

```rust
const SK: u64 = 0xA11CE;
let account_id  = derive_public_id(BaseElement::new(SK));      // cobra
let receiver_id = derive_public_id(BaseElement::new(0xB0B));   // destinatario
```

Son **identidades distintas**, y `an_authorized_claim_verifies` y
`the_rightful_recipient_can_claim` **verifican**. Si el circuito exigiera
que coincidieran, ese escenario no podría probar. Prueba.

### Qué demuestra el circuito, y qué no

Demuestra dos cosas ciertas por separado:

1. quien prueba tiene la clave de la cuenta que se acredita (`C_PK_CHECK`
   ata la identidad derivada a `COL_ACC_ID`), y
2. existe en el árbol un pendiente cuyo compromiso es
   `H(H(COL_R_ID, aleatorio), importe)`.

**No demuestra que `COL_R_ID` sea `COL_ACC_ID`.** `COL_R_ID` aparece cuatro
veces en el fichero —constante, construcción de la traza, la restricción
de constancia que resultó muerta (§38) y la lectura que arma el
compromiso— y ninguna aserción de frontera lo fija.

### Por qué la capa no lo salva

`client.rs:325` y `two_phase.rs:438` pasan `receiver.public_id` **como
ambos argumentos**, con el comentario *«cobrar es demostrar que el
pendiente estaba a su nombre»*. En operación honesta coinciden.

Pero **el diseño entero descansa en que la prueba se genera en el
cliente**: las funciones libres construyen la prueba con la clave de gasto
y la capa solo verifica. Construir la traza uno mismo no es un ataque
exótico —es el modo de operación con un cliente modificado—.

Es exactamente la forma de §8.1: *una propiedad impuesta por la capa y no
por el circuito, evitable por quien construya su propia traza*. Aquella
era el límite regulatorio; esta es la titularidad del cobro.

### Consecuencia

Quien conozca **posición, aleatorio e importe** de un pendiente puede
cobrarlo en su propia cuenta.

⚠️ **El pagador los conoce todos**: él eligió el aleatorio. Puede enviar
un pago y recuperarlo antes de que el receptor lo cobre.

Y el aviso viaja **fuera del mensaje ISO 20022**, por un canal lateral que
el propio proyecto declara sin especificar (§ de integración de estándares
del preprint de política). Cualquiera que lo intercepte tiene lo que hace
falta.

### Por qué no se vio antes

**El test que guardaba la propiedad no la prueba.**
`nobody_else_can_claim_a_pending_transfer` —cuyo comentario lo llama *«el
test que sostiene toda la segunda fase»*— cambia la **clave de gasto**, no
la identidad del receptor, así que falla por `C_PK_CHECK`. Su
documentación afirma que falla porque *«el compromiso reconstruido con su
identidad es otro»*, y eso no es lo que ocurre.

Y el que debía validarlo, `the_rightful_recipient_can_claim`, se llama *«el
destinatario legítimo»* sobre un escenario donde el destinatario **no** es
el titular de la cuenta. El nombre tapó el dato durante todo el desarrollo.

Es la cuarta vez que §16.5 se cobra una pieza en este proyecto, y esta vez
sobre la propiedad central de la vía de producción.

### Lo que hay que hacer

1. **Añadir la restricción que falta**: `COL_R_ID` debe ser `COL_ACC_ID`,
   impuesta en circuito.
2. **Corregir el escenario de test**, que hoy documenta el agujero en vez
   de detectarlo, y reescribir los dos tests para que prueben lo que dicen.
3. **Corregir los tres preprints**: describen el cobro como demostración de
   titularidad. Entrada 28 del backlog.
4. Revisar si `circuit_send` tiene una omisión análoga.

Hasta que 1 y 2 estén hechos, **la vía de dos fases no debe describirse
como segura frente a un cliente malicioso**.

### 39.1 Corregido y verificado

`C_PEND_IN` reconstruye el compromiso con **`COL_ACC_ID`**, la cuenta que
cobra, en vez de con `COL_R_ID`. Como `C_PK_CHECK` ya ata `COL_ACC_ID` a la
identidad derivada de la clave de gasto, **no queda hueco donde meter otra
identidad**. Es el mismo principio con que §4 cerró la fuga del saldo: la
propiedad va en la estructura, no en un test.

Pasar una identidad distinta ya no cuela por **dos** vías independientes: el
compromiso que `build_trace` coloca en la traza no llega a la raíz declarada,
y además viola la restricción.

**Los dos tests, rehechos.** El que decía sostener la segunda fase cambiaba
la clave y fallaba por `C_PK_CHECK`; ahora se llama
`without_the_account_key_nothing_can_be_claimed`, que es lo que prueba. Y
`nobody_else_can_claim_a_pending_transfer` es por fin el ataque real:
Mallory con **su propia cuenta y su propia clave válidas** intentando cobrar
un pendiente dirigido a otra identidad. Antes de esta corrección,
verificaba.

El escenario de test también se corrige: construía el pendiente a nombre de
`0xB0B` mientras cobraba la cuenta de `0xA11CE`, y así **documentaba el
agujero en vez de detectarlo**.

**Verificación**: `stark-experiment` 203 y `zk-ssl` 174, sin fallos.

### 39.2 Un fallo único, sin explicar

La primera ejecución después de aplicar la corrección dio **202 pasan, 1
falla**. Las **46 siguientes** —seis sueltas, veinte en paralelo y veinte con
un solo hilo— pasan las 203. No se capturó el nombre del test que falló.

Se investigó la hipótesis más plausible: 22 ficheros del crate manipulan el
**gancho de pánico del proceso** —estado global— en 32 puntos, y `cargo
test` ejecuta en paralelo. Las veinte pasadas con `--test-threads=1` no
mostraron diferencia con las veinte en paralelo: cero fallos en ambas. **La
hipótesis no queda respaldada.**

⚠️ Queda anotado sin adornar: **ocurrió una vez, no se ha reproducido en 46
ejecuciones, y la causa se desconoce.** Una batería que falla una de cada
cuarenta y siete sin explicación es una dependencia de confianza como
cualquier otra, y por eso figura aquí y no en una nota al pie.

### 39.3 `circuit_send` no tiene la omisión análoga

El punto 4 de la lista de arriba, comprobado.

En el envío la identidad del receptor **debe** ser libre —el pagador elige a
quién paga—, así que la pregunta análoga es otra: si el importe que se
debita está atado al que va dentro del compromiso. Lo está, por tres
piezas que comparten la misma columna:

| Pieza | Impone |
|---|---|
| `C_BALANCE` | `COL_BAL_NEW = COL_BAL − COL_AMT` — el débito |
| `C_PEND_VAL + 4` | el importe **dentro del compromiso** es `COL_AMT` |
| Aserción de frontera | `COL_AMT` en la fila 0 es la entrada pública |

Y `COL_AMT` figura en el vector `transport`, cuyas siete constancias ocupan
las ranuras **+0 a +6** — las que el solapamiento de §38 **no** pisa. El
límite regulatorio vuelve a imponerse en circuito, así que lo de §8.1 está
cerrado.

### 39.4 Qué alcance tienen entonces las ocho muertas de §38

Con la 27 corregida, el análisis queda así:

- En **`circuit_send`**, `COL_R_ID` y el aleatorio los **elige el pagador**.
  Que su constancia no se imponga no le da ninguna capacidad que no tuviera.
- En **`circuit_claim`**, `COL_R_ID` ya no lo lee nadie tras §39.1, y el
  aleatorio solo determina el compromiso en las filas donde se construye:
  si varía en otras, el compromiso sigue siendo el que sube al árbol y tiene
  que casar con la raíz declarada.

⚠️ **Esto es análisis, no demostración.** Hoy tres razonamientos de la
misma clase han resultado equivocados (§37.4, §37.6, y la atribución de
§37.7 que llevó hasta aquí). Se registra como lectura, no como garantía.

**El defecto de disposición sigue ahí y hay que arreglarlo**, aunque no se
le conozca consecuencia: ocho ranuras se calculan y se tiran, y un
comentario declara quince donde el reparto asigna siete. Dos vías:

| Vía | Coste | Riesgo |
|---|---|---|
| **Reasignar** los índices para que las ocho se impongan de verdad | `NUM_CONSTRAINTS` sube 8 y hay que rehacer la lista de grados | Alto: toca el espacio de índices entero |
| **Eliminarlas** y corregir el comentario | Trivial | Bajo, pero **decide** que no hacen falta apoyándose en el análisis de arriba |

⚠️ **La decisión se aplaza a propósito.** Ninguna de las dos es urgente
—no hay consecuencia conocida— y la primera es un refactor del espacio de
índices, que es exactamente donde este proyecto se ha hecho daño hoy. Se
elige con la cabeza despejada, no al final de una sesión de doce horas.

## 40. El barrido que §39 obligaba a hacer, y por qué no basta

§39 fue **una columna que el circuito lee y no ata a nada**, encontrada por
accidente. La pregunta obligada después no es corregir los papeles: es si
hay más. Esta sección registra el intento y su resultado, que es negativo.

### 40.1 Dos heurísticas, las dos fallidas

**«Columnas leídas sin aserción de frontera».** Devuelve entre 7 y 15 por
circuito, 119 en total. No sirve: casi ninguna columna se ancla por
aserción —se atan por restricciones—, así que la señal es todo ruido.

**«Columnas que aparecen pocas veces en el fichero»**, que era la firma
literal de `COL_R_ID` (cuatro apariciones). Devuelve **65 candidatas**, casi
todas estructurales: bits de camino, acumuladores de segmento, la clave de
gasto. Tampoco discrimina.

> **El defecto de §39 no tiene forma sintáctica.** Es semántico: *un valor
> que el probador elige, que entra en un compromiso, sin nada que lo ate a
> algo que no pueda elegir*. Ningún `grep` expresa eso.

Se registra el fracaso porque el proyecto ya sostiene que sus dos
herramientas caras no encontraron ninguno de los defectos reales. Esta es
la tercera, y tampoco.

### 40.2 Lo que sí acotó: tres circuitos construyen un compromiso

| Circuito | Estado |
|---|---|
| `circuit_claim` | El agujero de §39. **Corregido** (§39.1) |
| `circuit_send` | **Verificado sin omisión análoga** (§39.3) |
| `circuit_mint_pending` | Ver abajo |

### 40.3 Pregunta abierta en `circuit_mint_pending`

Verificado: **`MintPendingPublicInputs` no incluye la identidad del
destinatario**, y `get_assertions` no ancla `COL_R_ID`. Su constancia sí se
impone (grupo `C_TRANSPORT_NEW`, sin el solapamiento de §38).

La declaración pública dice, entonces, *«dos custodios autorizaron, el
suministro sube N respetando el tope, y un compromiso entra en el árbol de
pendientes»* — **sin nombrar a quién**.

⚠️ **No se afirma que sea un fallo.** Depende de qué cubra exactamente la
autorización de los custodios: si construir la prueba exige su cooperación,
el destinatario está cubierto por esa cooperación aunque no figure en la
declaración. Determinarlo pide leer el sub-circuito de umbral con el mismo
cuidado que llevó a §39, y **esa lectura no se ha hecho**.

Lo que sí se puede decir sin más trabajo: un supervisor que verifique esa
prueba **no aprende a quién se emitió**. Es coherente con el diseño de
privacidad y, a la vez, significa que la autorización de custodios **no es
auditable por destinatario**. Eso pertenece al cuadro de confianza residual
de los preprints, figure o no como defecto.

### 40.4 Lo que esto le hace a la auditoría externa

§39 es un defecto de una clase que **las herramientas del proyecto no pueden
detectar por construcción**: el detector de restricciones vacuas ve reaccionar
la ranura sobrescrita y la da por sana. Y el barrido sistemático de §40.1
no encuentra la clase.

Queda entonces un solo método conocido que sí la encuentra: **leer cada
circuito preguntando qué defiende cada comprobación e intentar lo que
debería impedir**. Es caro, es manual, y el autor acaba de demostrar sobre
sí mismo que se equivoca al hacerlo —tres veces en un día—.

> La auditoría externa deja de ser «deseable» y pasa a ser **el único
> instrumento conocido para una clase de defecto ya demostrada en este
> código**.

## 41. La autorización de custodios es posesión de claves, no aprobación

Respuesta a la pregunta que §40.3 dejó abierta. Se resolvió mirando **qué le
exige la capa a los custodios**, no leyendo el sub-circuito de umbral: más
barato y menos expuesto a equivocarse.

### 41.1 Lo verificado

```rust
pub struct ThresholdAuth {
    pub key_a: BaseElement, pub index_a: u64, pub path_a: CustodianPath,
    pub key_b: BaseElement, pub index_b: u64, pub path_b: CustodianPath,
}
```

**Claves de gasto en crudo.** Y las operaciones privilegiadas son métodos de
la capa —`mint`, `mint_to_pending`, `freeze`, `recovery`, `governance`— que
construyen la traza **dentro** (`mint.rs:48`, `two_phase.rs:559`).

`client.rs` ofrece prueba en la máquina del titular para `send` y `claim`
—y solo para eso—. **No hay vía equivalente para custodios.**

### 41.2 Las dos consecuencias

**Primera: la autorización no está ligada a la operación.** No hay firma
sobre unos parámetros: hay demostración de que dos claves del conjunto
participaron. Quien las tenga elige destinatario, importe y todo lo demás.
Eso responde a §40.3: el destinatario de una emisión a pendiente no figura
en la declaración pública **ni está cubierto por la autorización**; está
cubierto por la cooperación, que es otra cosa.

**Segunda, y más importante: las claves de custodio llegan al operador.**
Por construcción, no por descuido.

### 41.3 La asimetría, y qué dicen los papeles

El proyecto sostiene, con razón, que *la clave de gasto no necesita viajar
al operador*. Los preprints lo formulan con cuidado —*«no exige la entrega
de las claves de gasto **de los clientes**»*— y esa frase es cierta.

Pero el cuadro completo es este:

| Parte | Su clave | Prueba en |
|---|---|---|
| Titular de cuenta | **No sale de su máquina** | Su máquina (`client.rs`) |
| Custodio | **Se entrega a la capa** | El operador |

Es decir: **quienes conservan sus claves son quienes solo pueden mover su
propio dinero, y quienes las entregan son quienes pueden crearlo.** La
custodia de claves se concentra exactamente en la parte con más poder.

⚠️ Un operador que retenga dos claves de custodio puede emitir hasta el
tope, a cualquier destinatario, hasta agotar el cupo —y el cupo se renueva
rotando el conjunto—. El contador de intervenciones lo hace **visible**, que
no es poco, pero no lo impide.

### 41.4 Qué hacer con esto

No es un fallo de solidez: el circuito demuestra exactamente lo que dice
demostrar. Es **confianza residual no declarada**, y por eso pertenece al
cuadro de §5 de los preprints y a la tabla de §4.1 del de confianza
residual, donde hoy no está.

La corrección técnica —que los custodios prueben en su máquina, como los
titulares, y que la autorización cubra los parámetros de la operación— es
trabajo de diseño, no una línea. Queda como entrada de backlog.

## 42. Entrada 33: por qué «que los custodios prueben en su máquina» no es tan simple

Análisis de diseño, no implementación. Sale de §41, que estableció dos
cosas: la autorización de custodios es **posesión de claves**, y esas claves
**llegan al operador**.

### 42.1 El obstaculo que no se ve al enunciarlo

Para los titulares la solución existe y funciona: `client.rs` entrega
materiales, el titular prueba en su máquina, la capa verifica. **Una clave,
una máquina.**

Los custodios son **dos**, y el circuito actual demuestra conocimiento de
**ambas claves dentro de una sola traza**. Eso no se mueve «al cliente»
porque no hay un cliente: hay dos.

| Vía | Qué resuelve | Qué no |
|---|---|---|
| Un custodio prueba y el otro le pasa su clave | Nada | Reubica el problema: ahora la clave la tiene un custodio en vez del operador |
| Prueba multiparte (MPC sobre el probador) | Todo | Es investigación, no ingeniería de aplicación |
| **Firmas verificadas en circuito** | Las dos mitades | Cuesta, y no está medido |

### 42.2 La vía de las firmas, que resuelve las dos mitades a la vez

Cada custodio **firma un mensaje que cubre los parámetros de la
operación** —destinatario, importe, contador— en su máquina. El circuito
verifica dos firmas contra la raíz del conjunto de custodios, en vez de
demostrar conocimiento de dos claves.

Con eso:

- **Las claves no salen** de las máquinas de los custodios. Cierra la 32.
- **La autorización queda ligada a la operación.** Cierra la otra mitad de
  §41.2: hoy dos custodios autorizan «algo» y quien tenga las claves elige
  el qué; con firma sobre parámetros, autorizan **eso**.
- **Se vuelve auditable por operación**, que es lo que §40.3 echó en falta.

### 42.3 El coste, y una cifra que hay que retirar

§18 ya identifica el mismo primitivo por otra razón: *delegar la generación
de la prueba a un tercero exigiría verificar una firma en circuito*. Es
decir, **la entrada 21 y la 33 necesitan la misma pieza**, y construirla
una vez paga dos.

~~⚠️ **Retirada de una cifra sin fuente.** La entrada 21 del backlog decía
«~8.000 filas»... Esa cifra no figura en ninguna parte del repositorio... Se
retira.~~

⚠️⚠️ **ESTO ERA FALSO. Ver §42.5.** El «~8.000 filas» **sí** estaba en el
repositorio —`client.rs`, con el esquema concreto (Winternitz)—. Se retiró
por error, tras buscar donde no estaba. La cifra es legitima y se restituye.

### 42.5 ⚠️ Rectificacion: el ~8.000 tampoco era inventado

Igual que el 125,6 KB de §48.3, y por identica causa. §42.3 afirmo que
«~8.000 filas» no figuraba en el repositorio y lo retiro. **Estaba en
`client.rs`**:

> *«...verificar una firma dentro del circuito (Winternitz, ~8.000 filas
> adicionales).»*

Con esquema nombrado —Winternitz— y contexto —optimizacion para clientes
ligeros, no correccion de seguridad—. Es una **estimacion documentada**, no
una medicion exacta, pero no es una cifra de memoria: vive en el codigo.

**El mismo error, dos veces, en las mismas dos entradas (21 y 10).** Las dos
cifras que «retire por inventadas» eran reales; las dos las busque en
AUDITORIA/README/preprints y **no en los `.rs`**. Y peor: cite la retirada
del 8.000 como *precedente* para retirar el 125,6, encadenando el error.

> **Un barrido hereda los puntos ciegos de quien lo hace.** Busque las
> cifras donde yo esperaba que estuvieran. Las dos vivian donde no mire. Es,
> literalmente, la tesis de §40.4: ni las herramientas ni los barridos
> encuentran lo que su autor no sabe mirar. Solo la lectura completa —o un
> tercero— lo hace.

La entrada 21 recupera su cifra. La 10 estaba bien cerrada por otra via
(§48.3, la decision esta implementada), pero su cita del 8.000 como
«inventado» queda corregida aqui.

### 42.4 Lo que este análisis NO decide

No decide que se haga. Cambiar de «conocimiento de clave» a «firma sobre
mensaje» rehace el sub-circuito de umbral, toca `mint`, `mint_to_pending`,
`freeze`, `recovery` y `governance`, y exige elegir un esquema de firma
verificable en AIR. Es trabajo de diseño de semanas, y **lo primero sería
medir el coste del primitivo**, no escribirlo.

## 43. El reencuadre de la 33: no hacen falta firmas

§42 propuso verificar **firmas en circuito** para ligar la autorizacion de
custodios a la operacion. Al mirar como se autentica el sistema, esa via
resulta innecesaria — y ademas mas cara de lo que hace falta.

### 43.1 Como se autentica este sistema: sin firmas

No hay ningun esquema de firma en el proyecto. La identidad es
`derive_public_id(clave) = hash(dominio, clave)`, y autenticarse es
**demostrar conocimiento de la preimagen**: «conozco la clave cuyo hash es
esta identidad publica». Eso es todo lo que hacen `C_KEY_INPUT` y
`C_PK_CHECK`.

Los `grep` de «signature» o «firma» del codigo son la palabra en
comentarios, no verificacion criptografica de firmas.

### 43.2 El patron del titular ya es lo que los custodios necesitan

Un titular prueba «conozco la clave de esta cuenta» **en la misma traza**
donde estan el destinatario, el importe y el aleatorio (`COL_KEY`,
`COL_R_ID`, `COL_AMT`, todas presentes a la vez). La clave y el mensaje
conviven en una traza; que el titular no pueda cambiar el mensaje sin
romper su prueba es precisamente lo que se corrigio en §39.

Para los custodios el problema no es «faltan firmas» sino que **la clave se
entrega a la capa** en vez de quedarse en su maquina (§41). La solucion es
el mismo patron del titular, dos veces:

- cada custodio, en su maquina, produce los materiales que demuestran
  conocimiento de su clave sobre una traza que **ya incluye los parametros
  de la operacion**;
- la capa compone las dos mitades y verifica, sin ver ninguna clave.

**El primitivo ya esta construido y medido**: es el que corre en cada pago.
Lo que falta no es un esquema de firma nuevo, sino **reestructurar el
sub-circuito de umbral para que las dos mitades se prueben por separado** y
ligar el mensaje a la autorizacion — que es lo que ya se hace en el resto de
circuitos.

### 43.3 Correccion de §42.2 y §42.3

§42.2 presentaba «verificar firmas en circuito» como la via, y §42.3 hablaba
de medir el coste de ese primitivo. **Ambas cosas se corrigen**: no hay
primitivo nuevo que medir, porque la autenticacion por conocimiento de
preimagen es la que ya existe. La cifra retirada en §42.3 sigue retirada
—nunca estuvo medida—, pero la razon es mas fuerte: no habia nada que medir,
el coste ya esta en cada traza de pago.

⚠️ Lo que **si** es trabajo real: partir la prueba conjunta de dos claves
en dos pruebas componibles. Eso es reestructuracion de circuito, no un
primitivo que falte, y su coste es el de rehacer el umbral — no el de
inventar verificacion de firmas.

### 43.4 La leccion, otra vez la misma

§42 razono sobre un primitivo que el proyecto no usa —firmas— en vez de
mirar el que usa —conocimiento de preimagen—. Es la cuarta vez en la sesion
que un analisis se corrige al contrastarlo con lo que el codigo hace de
verdad (§37.4, §37.6, §40.1, y esta). El patron es consistente: **el error
esta siempre en razonar sobre lo que deberia haber, no sobre lo que hay.**

## 44. El precio real de la entrada 6, medido

§37.7 dejó el diagnóstico cerrado —posiciones con bits de camino no
constantes restauran los grados— y el pronóstico en «tiene arreglo, hay que
decidir el precio». Medido el precio, la decisión es más delicada de lo que
parecía.

### 44.1 Las dos caras baratas

**Capacidad: despreciable.** Los árboles de cuentas y pendientes tienen
profundidad 32 — 2^32 hojas. Reservar las primeras 255 posiciones es el
0,0000059 % de la capacidad. En congelados (profundidad 24) sigue siendo
inapreciable.

**Tests: coste acotado.** Reservar posiciones bajas rompe las aserciones
que hoy esperan que el primer pendiente caiga en la 0 — son de test, no de
produccion, y se corrigen con el escenario, como se hizo con §39.1.

### 44.2 La cara cara: `next_pending` se persiste y solo sube

`next_pending` es un contador persistido que **nunca reutiliza posiciones**
(`lib.rs:192`). Para un ledger **nuevo**, arrancarlo en 256 no cuesta nada:
ningun pendiente cae en el rango degenerado.

Para un ledger **existente**, no basta. Su `next_pending` ya vale algo bajo
y tiene pendientes vivos en las posiciones 0, 1, 2… Cambiar el punto de
arranque **no los mueve**, y esas posiciones siguen con los bits de camino
constantes. La correccion no es retroactiva sin **reubicar los pendientes
existentes** a posiciones altas.

⚠️ **Y reubicar un pendiente es mover valor en transito.** Un pendiente es
un pago enviado y no cobrado (§36). Cambiarlo de posicion recalcula la raiz
del arbol, y hacerlo mal —o a medias tras un fallo— pierde o duplica un
pago. Es exactamente la clase de operacion que §36 rodeo de verificacion:
no se hace en silencio, se verifica contra la raiz antes y despues.

### 44.3 Entonces la 6 no es «pagar un precio», es una migracion

La decision no es «reservar 256 posiciones: si o no». Es:

1. **Ledger nuevo**: `next_pending` arranca en 256. Trivial, sin coste.
2. **Ledger existente**: o se migran los pendientes vivos a posiciones
   altas —con la misma prudencia que §36—, o se acepta que **solo los
   ledgers nuevos** tienen los grados sanos y los viejos conviven con el
   modo depuracion roto hasta vaciarse.

La opcion 2b —correccion solo hacia delante— es defendible: el problema es
que la comprobacion de grados no protege en depuracion, no un fallo en
produccion, y un ledger viejo se vacia de pendientes al cobrarse todos. Pero
es una decision de politica, no una linea de codigo.

### 44.4 Por que se aplaza, con fundamento

No por cansancio: porque **decidir una migracion de pagos en transito es
del mismo peso que §36**, y §36 se hizo con una sesion dedicada, no como
paso trece de una larga. Lo que §44 aporta es que la decision quede **bien
planteada**: capacidad y tests son baratos, el nudo es el ledger existente,
y hay una salida hacia delante que no migra nada si se acepta su politica.

Las entradas 24 y 25 —el mismo grado dependiente del testigo en congelados,
cuentas y por valores del dominio— comparten este analisis: cuentas viven
en indices 0 y 1 por diseño, y ahi reservar no es opcion sin mover cuentas.
Para esas dos, la salida realista es la de §37.2 caso B: **declarar** que la
comprobacion de grados de winterfell es incompatible con grados que dependen
del valor, y documentarlo como limite de la herramienta.

## 45. La carrera del gancho de panico: reproducida y eliminada

§39.2 anoto un fallo unico de la bateria —202/1 una vez, 203/0 las 46
siguientes— y **descarto** la hipotesis del gancho de panico porque
`--test-threads=1` no cambiaba nada. Ese descarte estaba mal hecho.

### 45.1 El descarte de §39.2 no probaba la hipotesis

Con **un solo hilo no hay carrera posible**, asi que ver que con un hilo no
falla no dice nada sobre una carrera entre hilos. Se descarto una hipotesis
de concurrencia con un experimento que eliminaba la concurrencia. El
experimento correcto es el contrario: **subir** la contencion.

### 45.2 Reproducida subiendo la contencion

40 pasadas con `RUST_TEST_THREADS=16`: **una fallo** (la 38). La carrera es
real. No era 1 de 47 por azar: es 1 de ~40 cuando se fuerza el paralelismo.

### 45.3 El mecanismo

32 tests, en 22 ficheros, usan este patron:

```rust
let hook = take_hook();          // guarda el gancho global del PROCESO
set_hook(Box::new(|_| {}));      // lo sustituye por un silenciador
let r = catch_unwind(|| prove);  // provoca el panic esperado
set_hook(hook);                  // restaura
```

El gancho es **estado global del proceso**. Entre el `take_hook` de un test
y su `set_hook(hook)`, otro test en paralelo puede hacer su propio
`take_hook` y llevarse el **silenciador** en vez del gancho real. A partir
de ahi el estado queda corrompido y un panic que debia capturarse limpio
tumba un test — uno cualquiera, sin patron, no reproducible en secuencial.

### 45.4 El gancho no aportaba nada

El comentario de estos bloques dice que **el mensaje del panic se conserva**
porque winterfell da en el el detalle del fallo de restriccion, y que
descartarlo costo tres rondas (§25). Cierto — pero ese mensaje se extrae
del **payload del `Err`** que devuelve `catch_unwind`, con `downcast_ref`,
y eso es independiente del gancho. El gancho solo controla si Rust ademas
**imprime** el panic por stderr.

Es decir: el silenciador no protegía el dato —que ya estaba en el `Err`—,
solo callaba un `eprintln`, y a cambio metía la carrera. **Se elimina el
patron entero en los 32 sitios.** Los tests quedan identicos en correccion;
solo imprimen el backtrace del panic esperado, que es cosmetico.

> El codigo decía conservar el mensaje y a la vez lo silenciaba. El detalle
> viajaba por otra via, asi que el silenciador solo aportaba una carrera.

### 45.5 La leccion, sobre el metodo

§39.2 es la quinta vez en la sesion que un analisis se equivoca, y la unica
en un **experimento negativo mal diseñado** en vez de una atribucion. La
forma es la misma de siempre: se comprobo lo que era comodo comprobar —un
hilo— en vez de lo que ponía la hipotesis a prueba —muchos—. Un experimento
que no puede fallar si la hipotesis es cierta no la prueba.

### 45.6 La verificacion, con su propio tropiezo

El arreglo de §45 se registro en un commit (`bcb9f73`) **antes** de aplicar
el codigo: el parche no se habia descargado y el commit solo llevaba
AUDITORIA y BACKLOG. Durante unos minutos `main` afirmo una correccion que
no estaba en el codigo. Se corrigio hacia delante con `97d7c7f`, cuyo
mensaje nombra el commit anterior en vez de disimularlo.

Y hubo un segundo tropiezo instructivo. Tras aplicar el codigo de verdad,
una tanda de 40 pasadas dio **1 fallo** —justo lo que el arreglo debía
eliminar—. Antes de rediagnosticar en falso, se aisló la variable
sospechosa: ese fallo cayó en el arranque, con el build recién cambiado y
posiblemente a medio recompilar. Recompilando en frio primero, **260
pasadas a 16 hilos dieron cero fallos** (40 + 60 + 80 + 80), con el unico
fallo en aquella primera tanda sin recompilar.

⚠️ **Grado de certeza, explicito:** 260 pasadas sin fallo es evidencia
fuerte de que la carrera esta eliminada, no una demostracion. Una carrera
que aparecía 1 de 40 podria en principio aparecer 1 de 500; lo que se
afirma es que el mecanismo identificado en §45.3 se elimino y que la
reproduccion que lo delataba ya no ocurre en 260 intentos.

**La leccion se repite en el propio cierre:** el primer `1 de 40` post-fix
casi provoca un rediagnostico sobre una pasada contaminada por el build,
igual que el primer `0 de 40` pre-fix casi cierra la entrada sobre azar.
Las dos veces, la salida corria antes de que el estado fuera el que se
creía. Medir exige fijar primero lo que se mide.

## 46. La decision de la entrada 6, razonada

§44 dejo tres ramas de politica. Al bajar a decidir, el codigo corrige una
premisa y la eleccion se vuelve casi forzada.

### 46.1 Un dato de §44 que estaba mal: los huecos se reutilizan

`two_phase::allocate_pending` **reutiliza posiciones liberadas**:

```rust
for p in 0..self.next_pending {
    if !self.pending.is_occupied(p) { return Ok(p); }
}
Ok(self.next_pending)
```

§44 supuso que un ledger viejo «se vacia de pendientes al cobrarse todos» y
por eso la correccion solo-hacia-delante bastaba con el tiempo. **Falso.**
Cuando un pendiente en posicion baja se cobra, su hueco queda libre y la
capa lo **reasigna** al siguiente envio, que vuelve a caer en la posicion
degenerada. Un ledger no se cura solo: **recae**.

Esto **descarta la rama solo-hacia-delante** (§44.3, opcion 2b). No es que
sea peor: es que no existe. Mientras `allocate_pending` empiece en 0 y
reutilice huecos, siempre habra pagos en posiciones con bits de camino
constantes.

⚠️ **Y el comentario de `allocate_pending` se contradice a si mismo**,
como `circuit_send` en §1. Dice *«Nadie las reutilizaba»* en un parrafo y
dos mas abajo describe el reciclado como propiedad buscada: *«vale igual
para una posicion nueva que para una reciclada»*. El codigo reutiliza —el
bucle devuelve el primer hueco libre desde 0—, asi que la segunda frase es
la cierta y la primera es un resto rancio. Otro comentario que afirma lo
contrario de lo que hace la funcion doce lineas mas abajo. Se corrige al
redactar la 34.

### 46.2 Las tres ramas, contra los principios

**Migrar los pendientes vivos** (peso de §36). Correcta y completa, pero es
mover valor en transito, y su coste —una sesion dedicada, con verificacion
contra raiz— es real. **No aporta solidez de produccion**: el defecto que
arregla es que la *comprobacion de grados en depuracion* no protege. Gastar
una migracion de fondos para arreglar una comprobacion de tests es
desproporcionado. **Coherencia**: el precio no guarda proporcion con lo que
compra.

**Declararlo limite de winterfell** (como 24/25). La winterfell comprueba
en depuracion que el grado declarado se realice en la traza concreta; un
grado que depende del valor del testigo —y los bits de camino lo son— es
**incompatible con esa comprobacion por naturaleza**, no por un defecto del
circuito. §37.7 ya lo demostro: reformular no es posible sin cambiar las
posiciones, y cambiarlas arrastra la migracion. Es la misma situacion que
las cuentas en indices 0 y 1 (25) y que los valores de dominio (24): **una
propiedad de la herramienta, no un fallo del sistema.**

**No hacer nada sin declararlo.** Descartada de entrada: dejaria 65 tests
fallando en depuracion sin que el repositorio diga por que es aceptable.

### 46.3 La decision

**Se declara, no se migra.** Y se unifican 6, 24 y 25 bajo un mismo
enunciado, porque son el mismo fenomeno:

> La comprobacion de grados de winterfell en modo depuracion es
> incompatible con restricciones cuyo grado depende del valor del testigo.
> Los bits de camino de Merkle (arboles de cuentas, pendientes y
> congelados) y los margenes que pueden ser cero (tope de emision,
> diferencia de rango) tienen grado que colapsa en ciertos testigos
> validos. **No es un fallo de solidez** —en release, donde winterfell no
> comprueba grados, las pruebas se generan y verifican correctamente— sino
> el precio de arithmetizar en AIR una logica con ramas dependientes del
> dato. Se documenta como limite conocido; el modo release es el modo de
> produccion.

### 46.4 Por que esto es coherente y no una rendicion

El proyecto ya elige release como modo de produccion y lo documenta (§20).
La comprobacion de grados en depuracion es una red **adicional** de
winterfell, util donde aplica, y esta clase de restriccion queda fuera de
su alcance. Declararlo no rebaja ninguna garantia real: las pruebas
protegen lo mismo. Lo que cambia es que **el repositorio deja de tener 65
tests «rotos» sin explicacion** y pasa a tener un limite nombrado, que es
justo lo que distingue a este proyecto.

Lo que NO se hace, y se dice: no se persigue el 100 % de tests en
depuracion. Perseguirlo costaria una migracion de fondos (6) o seria
imposible (24, valores de dominio), y compraria una comprobacion que el
modo de produccion no necesita. **Coherencia sobre completitud.**

## 47. Diseno de la entrada 32/33: por que no es un parche, y por donde empieza

Se pidio resolver la 32 en una sesion. Lo honesto es especificar el diseno
hasta donde se puede verificar por lectura, y decir con precision donde
empieza el trabajo que exige compilar y medir. No entregar un circuito sin
probar sobre la creacion de dinero: seria el §39 de la proxima sesion, esta
vez sobre la emision.

### 47.1 Lo que hace hoy `circuit_threshold`, verificado

Toma **las dos claves en crudo** (`key_a`, `key_b`), deriva cada identidad
con `hash(CUSTODIAN_DOMAIN, key)`, y **sube las dos por el arbol de
custodios en una sola traza de dos carriles** (`LANE_B`), demostrando que
ambas hojas pertenecen a la raiz del conjunto. La autenticacion **es**
conocimiento de las dos preimagenes, probado a la vez.

Por eso las claves llegan al operador (§41): construir esa traza unica
exige tener las dos claves en la misma maquina, y esa maquina es la capa.

### 47.2 Que hay que cambiar, y el obstaculo real

Para que las claves no salgan de las maquinas de los custodios, la traza de
dos carriles tiene que **partirse en dos pruebas independientes** —una por
custodio, cada una demostrando conocimiento de UNA preimagen y pertenencia
de UNA hoja— y luego **componerse** en la evidencia de que el umbral se
cumplio.

⚠️ **La composicion de pruebas no existe en este proyecto.** Barrido hecho:
no hay recursion, folding, agregacion ni verificacion-de-prueba-en-circuito
en ningun crate. La unica mencion (`Proof::from_bytes` en el puente ISO) es
deserializacion, no composicion.

Esto es lo que convierte la 32 en trabajo de semanas y no en un parche: no
es reescribir `circuit_threshold`, es **construir un mecanismo que el
proyecto no tiene**.

### 47.3 Las dos vias de composicion, con su coste

**Via A — verificar dos pruebas en un circuito (recursion).** Cada custodio
produce una prueba STARK de su mitad; un circuito verificador comprueba las
dos. Es el enfoque general, y el mas caro: verificar una prueba STARK dentro
de un circuito AIR es de los problemas mas pesados del area, y winterfell no
trae verificador recursivo de serie. Coste probable: alto, y **no medido**.

**Via B — firma sobre el mensaje, compuesta trivialmente.** Cada custodio
firma en su maquina un mensaje que cubre los parametros de la operacion
(destinatario, importe, contador), demostrando conocimiento de su clave por
el mismo primitivo de preimagen que ya corre (§43). La capa **recoge las dos
pruebas y las verifica por separado** contra la raiz del conjunto —no hay
que componerlas en una sola, basta con exigir las dos—. Cierra ademas la
otra mitad de §41.2: la autorizacion queda **ligada a la operacion**, no a
«algo».

La via B es mas simple y no necesita recursion. Su trabajo real: separar los
dos carriles de `circuit_threshold` en dos invocaciones y anadir el mensaje
como entrada atada a la prueba de cada custodio. Es reestructuracion de un
circuito que ya existe, no un primitivo nuevo.

### 47.4 La decision de diseño, hasta donde se puede tomar por lectura

**Via B.** Razones, todas verificables:

1. El primitivo que necesita —conocimiento de preimagen con mensaje en la
   traza— **ya corre en cada pago** (`circuit_send`, `circuit_claim`). La
   via A necesita un verificador recursivo que no existe.
2. La via B no compone pruebas: la capa exige dos, independientes. Eso evita
   por completo el obstaculo de §47.2.
3. Liga la autorizacion a la operacion, que es la otra mitad del problema.

⚠️ **Lo que esta decision NO es:** una implementacion. Separar los dos
carriles de `circuit_threshold`, definir el mensaje y atarlo, y rehacer los
tests de `mint`, `mint_to_pending`, `freeze`, `recovery` y `governance`
—que todos usan `ThresholdAuth`— es trabajo que **exige compilar y medir**, y
este asistente no puede verificar un cambio de solidez sobre la creacion de
dinero sin ejecutarlo. Se deja el diseño, no el codigo.

### 47.5 El primer paso concreto de la implementacion

No es escribir el circuito. Es un **experimento acotado y medible**, del
estilo de los que han funcionado en esta auditoria:

> Separar UN carril de `circuit_threshold` en una prueba independiente que
> demuestre conocimiento de una clave y pertenencia de una hoja, sin tocar
> el otro carril ni los cinco circuitos que consumen `ThresholdAuth`.
> Medir: numero de restricciones, filas, tiempo de prueba. Si un carril
> solo se sostiene, la via B es viable y el coste esta medido. Si no,
> el diseño necesita revision antes de tocar produccion.

Eso es una sesion de trabajo con toolchain, no un parche a ciegas, y es el
punto por el que empieza la 33.

## 48. La entrada 10, y otra cifra sin fuente

La entrada 10 pedia «hacer explicita la eleccion» sobre la brecha entre
seguridad conjeturada (127 bits) y demostrable, diciendo que el coste
estaba medido: **36,7 KB → 125,6 KB**. Al verificar para cerrarla, dos
cosas.

### 48.1 El hecho SI esta declarado

La brecha figura en el README (lista de hallazgos, punto 4: *«127 bits
conviven con 29-63»*) y en el preprint comparativo (Finding 3: parametros
rapidos anuncian seguridad conjeturada alta y entregan mucha menos
demostrable salvo que se refuercen extension de campo y consultas, lo que
sube el tamano de prueba). Como enunciado, no falta.

### 48.2 La decision NO esta tomada, y la cifra de su coste no existe

Lo que la entrada 10 pide —**elegir** entre cerrar la brecha o aceptarla
conjeturada— no esta resuelto en ninguna parte. El README la lista como
hallazgo, no como decision.

⚠️ **Y la cifra que la haria decidible no esta medida.** El backlog daba
«36,7 KB → 125,6 KB». `36,7 KB` existe: es el tamano de prueba STARK
**actual** (README, tabla de backends). Pero **`125,6 KB` no aparece en
ningun punto del repositorio**, ni `29 bits` como cota demostrable
concreta. Es, casi con seguridad, otra cifra escrita de memoria al redactar
la lista —como el «~8.000 filas» de la 21, retirado en §42.3—.

Se retira. El coste real de cerrar la brecha —reforzar la extension de campo
y las consultas, y medir el tamano de prueba resultante— **no se ha medido**,
y hasta que se mida no debe citarse ninguno.

### 48.3 ⚠️ Rectificacion: la cifra SI existia, y la decision esta implementada

Lo de §48.2 esta **mal**, y se corrige aqui con el mismo estandar que el
resto de la sesion. Retire «125,6 KB» por «sin fuente» tras buscar en README,
AUDITORIA y preprints. **No busque en el codigo del puente ISO**, y ahi
estaba:

`iso_bridge.rs` documenta que su configuracion por defecto —**120 queries,
blowup 16**, frente a los 32/8 de los circuitos normales— **alcanza 128 bits
de seguridad DEMOSTRABLE** (no conjeturada), medida en `compliance_real_proof`,
y **cuesta ~125 KB y ~45 ms**. De ahi salio el 125,6.

Es decir: la decision de la 10 **no solo esta tomada, esta implementada**. El
proyecto tiene una config de 128 bits demostrables, la usa por defecto en el
puente, y deja la eleccion explicita: *«quien quiera pruebas mas pequenas
debe pedirlo y saber que cambia»*. Exactamente lo que la entrada 10 pedia.

⚠️ **Mi error fue el de siempre**: afirmar «no existe» tras una busqueda
incompleta, que es la misma forma que §37.4 y §40. Un «no lo encuentro» no es
un «no existe». La leccion se cobra una vez mas, ahora sobre una correccion
mia.

La cifra 36,7 KB (proof STARK normal) frente a ~125 KB (128 bits
demostrables) es el coste real de la garantia fuerte, **medido y en el
repositorio**. La decision de que circuitos usan cual queda como afinado, no
como frente abierto.

## 49. La entrada 30 se parte en dos, y una mitad huele a solidez

Al bajar a decidir la disposicion de las ocho ranuras solapadas (§38), el
codigo separa dos casos que el enunciado unico ocultaba. Uno es cosmetico.
El otro **puede** ser un fallo de solidez latente, y por eso la 30 no se
aplica en esta sesion.

### 49.1 En `circuit_claim`: borrar, no reasignar

Las ocho restricciones pisadas imponen la constancia de `COL_R_ID`
(identidad del receptor) y `COL_SALT` (aleatorio) entre filas. Tras §39.1,
`circuit_claim` **ya no usa `COL_R_ID`** para nada: el compromiso se
reconstruye con `COL_ACC_ID`. Esas ocho no protegen nada aqui —lo dijo ya
§39.4—, asi que la via correcta no es reasignar indices para salvarlas, es
**borrarlas y corregir el comentario** (que declara 15 donde el reparto da
7). Barato y seguro.

### 49.2 En `circuit_send`: puede ser un §39 latente

En el envio la identidad del receptor **es libre** —el pagador elige a quien
paga— y `C_PEND_IN` **la lee** para construir el compromiso
(`result[C_PEND_IN + 4 + i] = pend_in * (next[..] - current[COL_R_ID + i])`,
lineas 941-942). Aqui la constancia de `COL_R_ID` entre filas **si importa**:
si esta muerta por el solapamiento, un probador podria poner una identidad
en la fila donde se arma el compromiso y otra distinta en el resto de la
traza.

⚠️ **No se afirma que sea explotable.** Igual que en §38.2 antes de
confirmar §39: podria estar cubierta por otra via —que la subida al arbol
ate el compromiso a algo constante, o que la fila del compromiso sea la
unica que cuenta—. Pero es **exactamente la pregunta que §39 enseño a no
responder por lectura**: hay que construir un testigo con dos identidades y
ver si verifica.

### 49.3 Por que la 30 no se cierra hoy

Porque su mitad `send` es una hipotesis de solidez sobre la **creacion de
compromisos de pago**, y este proyecto —y este auditor— ya ha aprendido dos
veces esta semana que esas no se resuelven a ojo: §39 salio de un test
discriminante, no de mirar. Aplicar un borrado o una reasignacion en `send`
sin ese test es la clase de acto que sembro el §39 original.

**El primer paso, con toolchain, es un test discriminante** al estilo de
§39: en `circuit_send`, construir una traza donde `COL_R_ID` valga una
identidad en la fila del compromiso y otra en el resto, y ver si la prueba
verifica. Si verifica, es §39 en el envio y es urgente. Si se rechaza,
existe una via que lo ata, se documenta, y entonces la 30 es cosmetica y se
resuelve borrando en los dos circuitos.

La 30 queda **partida**: la mitad `claim` lista para un borrado seguro, la
mitad `send` elevada a pregunta de solidez con test pre-especificado.

## 50. ⚠️ La mitad `send` de la 30 ES un fallo de solidez

El test discriminante de §49.2 **verifica**: `circuit_send` acepta una traza
donde `COL_R_ID` vale una identidad en la fila del compromiso
(`ROW_FROZEN_ROOT`) y otra distinta en las otras 743 filas. Confirmado el
30-07-2026 por `a_send_with_inconsistent_receiver_identity_is_rejected`, que
salta con el mensaje SOLIDEZ.

### 50.1 Lo confirmado

La restriccion que impone la **constancia de `COL_R_ID` entre filas** —una de
las ocho que el solapamiento de §38 sobrescribe— esta **muerta**. El circuito
no ata el valor de `COL_R_ID` que usa el compromiso (`ROW_FROZEN_ROOT`, donde
`C_PEND_IN` lee) al valor que lleva el resto de la traza.

Que esa restriccion estaba muerta se sabia desde §38. Lo que §50 anade es que
**en `circuit_send` esa constancia SI protegia algo**, a diferencia de
`circuit_claim` (donde §39.1 dejo `COL_R_ID` sin uso). En `send` el receptor
es libre, y su constancia era lo unico que ataba las filas.

### 50.2 Lo que aun NO esta confirmado, y no se sobreafirma

El test demuestra que **las filas no estan atadas**. No demuestra todavia el
ataque completo —que un atacante coloque SU identidad en la fila del
compromiso y cobre un pago ajeno—, porque el test dejo la fila del compromiso
con la identidad de la victima. Queda un segundo test pre-especificado:
corromper `ROW_FROZEN_ROOT` con la identidad del atacante y ver si el
compromiso resultante puede reclamarse.

⚠️ Pero la propiedad rota ya es grave por si sola: **un circuito que no ata
una columna de identidad entre las filas que la usan no demuestra lo que
dice**. Si el compromiso se construye en una fila y la constancia con el
resto no se impone, cualquier razonamiento que dependa de «la identidad del
receptor es unica en la traza» es falso. Eso basta para tratarlo como fallo.

### 50.3 Relacion con §39

Es el mismo fenomeno que §39, en el circuito hermano, y **la misma raiz**: el
solapamiento de §38. §39 fue `circuit_claim` no atando el compromiso a la
cuenta; §50 es `circuit_send` no atando la identidad del compromiso al resto
de su propia traza. Los dos vienen de restricciones que se creian impuestas
y no lo estaban —uno por omision (§39), otro por sobrescritura (§38/§50)—.

Y confirma lo que §39.4 dejo como sospecha: la disposicion de las ocho
ranuras **no era cosmetica en `send`**. Se catalogo como aplazable; era un
fallo de solidez esperando.

### 50.4 Que hay que hacer, y el orden

1. **Reasignar los indices** de `circuit_send` para que las ocho
   restricciones —en particular la constancia de `COL_R_ID` y `COL_SALT`—
   se impongan de verdad. Esto es el refactor del espacio de restricciones
   que §39.4 temia, pero ahora es **obligatorio**, no opcional.
2. **Rehacer la lista de grados** de `send` en consecuencia.
3. **El segundo test** (§50.2) para caracterizar el alcance completo.
4. Revisar si `circuit_mint_pending` —el tercer constructor de compromisos
   (§40.2)— tiene la constancia viva o tambien pisada.

⚠️ Exige toolchain y una sesion dedicada: es cirugia en el circuito de
creacion de pagos, y hacerla a ciegas seria sembrar el proximo fallo. Pero
la **prioridad cambia**: la 30 pasa de aplazada a la cabeza de solidez, junto
a lo que quede de la 32.

### 50.5 Corregido y verificado

El arreglo, el 30-07-2026: **`C_TRANSPORT` recibe las 15 ranuras que su
codigo escribe** (era 7). El unico cambio de fondo es `C_ID_CONST =
C_TRANSPORT + 15` en vez de `+7`; el resto de constantes se desplaza en
cascada y `NUM_CONSTRAINTS` sube 8. La lista de grados ajusta su bloque de
grado-1-sin-ciclo de 13 a 21 entradas (saldo 1 + suministro 1 + transporte
15 + `C_ID_CONST` 4).

**Las 8 restricciones no se escribieron: ya existian.** Estaban en
`evaluate_transition` —`next[COL_R_ID + i] - current[COL_R_ID + i]` y lo
mismo para `COL_SALT`— desde siempre, cayendo en indices que `C_ID_CONST`,
`C_SBIT_BOOL` y `C_FIRST_S` sobrescribian. Darles sitio propio basto para
imponerlas.

**Verificacion**: el test `a_send_with_inconsistent_receiver_identity_is_rejected`,
que hasta hoy verificaba (fallo) y estaba en `#[ignore]`, **ahora pasa**: la
traza con `COL_R_ID` inconsistente entre la fila del compromiso y el resto
**se rechaza**. `stark-experiment` pasa de 203 a **204** (el testigo sale de
ignore y entra en verde), 0 fallos. La constancia se impone.

⚠️ **Lo que este arreglo cierra y lo que no.** Cierra la mitad `send` de la
30: la identidad del receptor esta atada a traves de la traza. Queda de §50.4:
(1) el segundo test de §50.2 —caracterizar el ataque completo— es ahora
innecesario como urgencia, porque la propiedad esta impuesta, pero vale como
regresion; (2) **revisar `circuit_mint_pending`**, el tercer constructor de
compromisos, sigue pendiente: no se ha verificado que su constancia no este
igualmente pisada. Y la mitad `claim` (borrar las 8 muertas, que alli sobran
tras §39.1) queda como limpieza sin urgencia.

### 50.6 La 35: `circuit_mint_pending` esta sano (el fallo no era sistemico)

Tras §50, quedaba la pregunta de si el tercer constructor de compromisos
—`circuit_mint_pending`— tenia el mismo solapamiento. Se respondio como en
§50: con un test discriminante, no por lectura.

**Resultado: rechaza.** `a_mint_pending_with_inconsistent_receiver_identity_is_rejected`
construye una traza con `COL_R_ID` distinto entre la fila del compromiso
(`ROW_ROOT`) y el resto, y **la prueba no verifica**. La constancia esta
viva. Confirmado ademas que rechaza por la razon correcta (§16.5): la
disposicion de `mint_pending` **cuadra de verdad** —`C_TRANSPORT_NEW`
reserva sus 12 ranuras (transporte 4 + identidad 4 + aleatorio 4) y el
grupo siguiente arranca limpio en +12—, a diferencia de `send`, donde
`C_TRANSPORT` declaraba 15 pero solo tenia 7 antes del grupo siguiente.

La cadena que lo ata: `C_PEND_IN` lee `COL_R_ID` para el compromiso, y la
constancia de `C_TRANSPORT_NEW + 4` garantiza que ese valor es el mismo en
toda la traza. Corromper una fila rompe la cadena donde la constancia lo
prohibe.

**Lo que esto decia del fallo de §50** —y que §50.7 corrige—: se afirmo
aqui que el fallo «no era sistemico» y que `mint_pending` demostraba que no
habia defecto comun. Eso resulto **precipitado**: la 36 destapo despues que
`claim` **tambien** tenia el solapamiento (§50.7), sobre el aleatorio. De los
tres constructores, `mint_pending` es el unico con la disposicion bien
contada; `claim` y `send` la tenian mal. El test de `mint_pending` queda como
**regresion permanente** y su veredicto (sano) sigue en pie; lo que se retira
es la generalizacion «no sistemico», otra vez el error de concluir de un caso.

## 50.7 ⚠️ La 36 no era limpieza: tercer fallo, en `claim` sobre el aleatorio

La 36 se catalogo —por el asistente, tres veces— como «borrar 8 restricciones
muertas de `claim`, borrado seguro». Al bajar a hacerlo, el codigo la
desmintio: **`claim` tiene el mismo solapamiento que `send`** (`C_TRANSPORT`
declara 15, `C_ID_CONST` arranca en +7). Tras §39.1 `COL_R_ID` esta muerto de
verdad y su constancia sobra —eso era cierto—, **pero el compromiso aun lee
`COL_SALT`** (`C_PEND_IN + 8`), y la constancia de `COL_SALT` era otra de las
ocho pisadas.

**Confirmado por test** (`a_claim_with_inconsistent_salt_is_rejected`): una
traza con `COL_SALT` distinto entre la fila del compromiso (`ROW_FROZEN_ROOT`)
y el resto **verificaba**. Es el mismo fallo que §50, en el tercer circuito,
sobre el aleatorio en vez de la identidad.

**Corregido igual que §50.5**: `C_TRANSPORT` recibe sus 15 ranuras (`+15` en
vez de `+7`), la lista de grados pasa de 13 a 21. Las 8 restricciones ya
estaban escritas; darles sitio impone la constancia. El test pasa de rojo
(ignore) a **verde**: 205 tests, 0 fallos.

### 50.7.1 La leccion, la septima y la mas incomoda

La 36 era el error del dia en su forma mas pura: yo la habia degradado a
«cosmetica» **tres veces** —en §39.4, en §49.1 y al cerrar la 35— sin mirar
que el compromiso seguia leyendo el aleatorio. Cada vez razone «claim ya no
usa COL_R_ID, luego las 8 sobran», y cada vez me salte que `COL_SALT` no es
`COL_R_ID`. Un fallo de solidez en el circuito de cobro vivio tres etiquetas
de «seguro» hasta que un test discriminante lo miro.

Y corrige el balance: **los tres constructores de compromisos tenian el
solapamiento** —no «dos de tres» como dijo §50.6—. Lo que variaba era cual
columna pisada seguia viva: identidad en `send`, aleatorio en `claim`,
ninguna con consecuencia en `mint_pending` (su disposicion, ademas, estaba
bien). El defecto SI era comun; su explotabilidad, no.

## 51. La 33, preparada: inventario y especificacion del primer experimento

La implementacion de la 32/33 (via B, §47) empieza por el experimento de
§47.5: separar un carril de `circuit_threshold` y medirlo. Antes de escribir
codigo —que exige toolchain y no se hace a ciegas en el circuito de creacion
de dinero— se deja aqui el punto de partida leido y el plano del experimento.

### 51.1 Inventario: como esta hecho `circuit_threshold` hoy

Es un circuito de **dos carriles simetricos**, A y B, uno por custodio.
`LANE_B = STATE_WIDTH = 12` desplaza el carril B dentro de una traza de
`TRACE_WIDTH = 34` columnas y `TRACE_LENGTH = 64` filas.

**Columnas por carril** (A / B): estado del hash (0..12 / 12..24), bit de
camino (`COL_BIT_A`=24 / `COL_BIT_B`=25), clave (`COL_KEY_A`=26 / 27), indice
(`COL_IDX_A`=28 / 29), acumulador de indice (`COL_ACC_A`=30 / 31). Columnas
**compartidas**: `COL_SBIT`=32 y `COL_SACC`=33, el bit y el acumulador del
**segmento de rango** que impone el orden.

**Cada carril, por separado, ya prueba lo que un experimento de un carril
necesita**: conocimiento de la clave (`C_KEY_INPUT`), derivacion de la hoja
por el hash Rescue (`C_HASH_*`, `C_CAP_*`), y pertenencia al arbol subiendo
por los bits de camino (`C_PLACE_*`, `C_BIT_BOOL`, `C_TRANSPORT`), hasta la
raiz comun en `ROW_ROOT`=39.

### 51.2 El acoplamiento: donde A y B se necesitan

Solo hay **un** punto donde los dos carriles se cruzan de verdad, y es el
que hay que cortar. El segmento de rango (`C_HORNER`, `C_SEG_LINK`,
`COL_SACC`) reconstruye por descomposicion en bits **tres** valores
(linea 594):

```rust
expected = [ IDX_A,  IDX_B,  IDX_B - IDX_A - 1 ]
```

Los dos primeros son de cada carril. El tercero, `IDX_B - IDX_A - 1`, es el
**unico** que necesita ambos indices a la vez: impone *A antes que B, sin
repeticion*, y es lo que garantiza que las dos firmas son de custodios
**distintos**. Todo lo demas en el circuito es dos copias independientes.

### 51.3 Especificacion del experimento (parte A de §47.5)

**Objetivo**: un `circuit_threshold_single` que tome UNA clave, demuestre
conocimiento de su preimagen y pertenencia de su hoja al arbol de custodios,
y genere una prueba STARK autonoma. Medir su coste. Si un carril solo se
sostiene con coste razonable, la via B es viable.

**Traza**: una sola copia del carril A. `TRACE_WIDTH` baja de 34 a ~14
(estado 12 + bit + clave + indice + acc, sin las columnas `_B` ni las
compartidas de rango). `TRACE_LENGTH` = 64 se mantiene (la subida al arbol
no cambia).

**Restricciones a conservar** (del carril A): `C_HASH_A`, `C_CAP_A`,
`C_PLACE_A`, `C_BIT_BOOL` (solo el bit A), `C_KEY_INPUT` (solo A), `C_ACC`,
`C_ACC_FINAL` (solo A), `C_TRANSPORT`.

**Restricciones a ELIMINAR** (el acoplamiento y el carril B entero): todo
`*_B`, y **todo el segmento de rango** (`C_SBIT_BOOL`, `C_FIRST_S`,
`C_HORNER`, `C_SEG_LINK`, columnas `COL_SBIT`/`COL_SACC`). El orden estricto
**no existe con un solo custodio** — no hay dos indices que ordenar.

**Entradas publicas**: la raiz del arbol de custodios (igual que hoy) y el
`derive_public_id` de la clave. Se elimina la mitad B.

⚠️ **Punto de diseño que el experimento debe resolver, no dar por hecho**:
como se combinan DOS pruebas single en la evidencia de umbral. La via B dice
«la capa verifica las dos por separado y exige ambas» (§47.3), pero **el
orden estricto —que eran custodios distintos— vivia en el circuito conjunto
y desaparece al separar**. Con dos pruebas independientes hay que reimponerlo
fuera: la capa debe comprobar que las dos hojas probadas tienen indices
distintos, o el umbral acepta la misma firma dos veces. **Esto es lo que el
experimento tiene que medir y decidir**: donde vive el orden estricto cuando
ya no hay traza conjunta. Es la pregunta abierta real de la 33, y §47 no la
cerro.

### 51.4 Metrica de exito del experimento

Generar y verificar la prueba de un carril, y medir: `NUM_CONSTRAINTS`,
filas efectivas, tiempo de prueba y tamano. Comparar con el circuito conjunto
actual. **Criterio**: si dos pruebas single cuestan aproximadamente lo mismo
que una conjunta y el orden estricto se puede reimponer en la capa sin volver
a acoplarlas, la via B es viable y se implementa. Si cuesta mucho mas, o si
reimponer el orden exige recomponer las pruebas —lo que el proyecto no sabe
hacer (§47.2)—, el diseño necesita revision antes de tocar los cinco
circuitos que consumen `ThresholdAuth`.

## 52. El experimento de la 33, ejecutado: la via B es viable

§47.5 pedia separar un carril de `circuit_threshold` y medirlo. Hecho el
30-07-2026, en **dos variantes** —porque §51.3 dejo abierto como se reimpone
«custodios distintos» al desaparecer el orden estricto—.

### 52.1 La medicion

| circuito | cols | filas | restr. | ms | bytes |
|---|---|---|---|---|---|
| A single (indice publico) | 16 | 64 | 26 | 4,1 | 14.700 |
| B single (nulificador) | **14** | 64 | 35 | 4,1 | 15.293 |
| conjunto (dos carriles) | 34 | 64 | 60 | 5,1 | 20.122 |

**Dos pruebas single: 8,2 ms y ~30 KB. Una conjunta: 5,1 ms y 20 KB.**

El criterio de §51.4 era «si dos single cuestan aproximadamente lo mismo que
una conjunta». El resultado es **1,6× en tiempo y 1,5× en tamano**: no es lo
mismo, pero es un factor constante pequeño sobre operaciones **raras**
—emision, congelacion, gobernanza—. Ocho milisegundos y treinta kilobytes no
son un coste que decida nada.

✅ **Veredicto: la via B es viable, y esta medida.** Compra que los custodios
no entreguen nunca su clave (la 32) por un sobrecoste despreciable.

### 52.2 Un resultado que no se esperaba

La variante que **protege la privacidad sale mas estrecha**: 14 columnas
frente a 16, pese a incluir un hash entero de mas. Al no publicar el indice
no hace falta atarlo, y desaparecen `COL_IDX`, `COL_ACC`, el acumulador de
bits y su comprobacion final. Mas restricciones (35 frente a 26) pero traza
mas estrecha, y **el tiempo es identico**.

Es decir: **elegir la variante privada no cuesta rendimiento**. La eleccion
entre A y B es puramente de modelo de confianza, sin compensacion tecnica
que la enturbie.

### 52.3 Recomendacion: variante B, y el argumento no es mio

`circuit_threshold` **ya declara hoy**, en el comentario de sus entradas
publicas:

> *«Los indices y las claves de quienes firman son privados: se sabe que dos
> custodios distintos del conjunto autorizaron, pero no cuales.»*

Es una propiedad **elegida y documentada** del sistema. La variante A la
rompe: publicaria que custodios firmaron cada emision. Arreglar la 32 —que
las claves no lleguen al operador— **no debe costar una propiedad de
privacidad distinta que el proyecto ya habia decidido tener**. Cambiarla en
silencio, y ademas sin tocar los preprints que la describen, seria
exactamente la clase de deriva que esta auditoria existe para impedir.

Por eso: **variante B**. Preserva lo que hay, arregla lo que falla, y no
cuesta mas.

### 52.4 Lo que B NO conserva, dicho antes de que lo descubra otro

El circuito conjunto de hoy no revela **nada** sobre los firmantes, ni
siquiera si son los mismos entre dos operaciones. B publica un nulificador
**estable por custodio**, asi que un observador puede agrupar: *«el custodio
desconocido X autorizo estas cinco emisiones»*. No sabe quien es X, pero sabe
que es el mismo.

⚠️ **B es por tanto mejor que A pero peor que el conjunto actual** en
privacidad. La escala honesta es:

| | claves al operador | firmantes identificables | enlazables |
|---|---|---|---|
| conjunto (hoy) | ❌ **si (fallo 32)** | no | no |
| A single | no | **si** | si |
| B single | no | no | **si** |

Cerrar la enlazabilidad es atar el nulificador tambien al identificador de
la operacion —`H(dominio, clave, operacion)`—, que **hace falta de todos
modos** para la otra mitad de la 33: que la autorizacion cubra los
parametros (§41.4). B no es solo la mejor de las dos: esta en el camino de
la solucion completa.

### 52.5 Si la institucion quisiera lo contrario

Hay un argumento legitimo para A que no se oculta: un banco central podria
**querer** registro auditable de que dos custodios autorizaron cada emision,
por rendicion de cuentas. Si esa fuera la exigencia, A es lo correcto — pero
entonces es un **cambio declarado del modelo de confianza**, que hay que
escribir en los preprints (la 28) y no colar como detalle de implementacion.
La recomendacion es B; la decision, del proyecto.

### 52.6 Dos errores mios en la medicion, registrados

**Uno.** El test de §16.5 sobre el indice esperaba que la traza invalida
fallara **al probar**. Fallo: en **release winterfell no comprueba las
restricciones al generar la prueba** —esa es la comprobacion de depuracion
de §20—, asi que el probador emite la prueba y es el **verificador** quien la
tumba. El motivo del rechazo era el correcto; la etapa que predije, no.

**Dos.** Puse **29** restricciones para el circuito conjunto en la tabla,
calculado a ojo como «26 + 3». El real, contando su cadena de constantes, es
**60**. Otra cifra sin verificar, la misma clase que §42.5 y §48.3, y en la
misma sesion en que las retire. Corregida.

Las dos las caza el mismo metodo de siempre: escribir la comprobacion y
dejar que el codigo conteste.

### 52.7 Decision tomada: variante B

El 30-07-2026 el proyecto elige la **variante B** (nulificador derivado de
la clave). Razon, en una linea: **arreglar la 32 no puede costar el
anonimato dentro del conjunto que `circuit_threshold` ya declaraba**, y no
cuesta —B es mas estrecha y tarda lo mismo (§52.2)—.

Queda por tanto **descartada la variante A**. Su codigo **no se borra**: es
la comparacion medida que sostiene §52.1, y esta convencion —marcar en vez
de borrar— es la del resto del proyecto. Pero se marca en su cabecera como
NO ELEGIDA, porque un circuito de autenticacion de custodios vivo y sin
marcar es una invitacion a cablearlo por error.

⚠️ **Lo que esta decision NO cierra.** La 33 sigue abierta: elegir la
variante no es implementarla. Falta (a) atar el nulificador al identificador
de la operacion, que cierra la enlazabilidad de §52.4 **y** es la otra mitad
de la 33 (§41.4); (b) la comprobacion en la capa de que los dos
nulificadores difieren —sin ella el umbral 2-de-N es 1-de-N—; y (c)
sustituir `ThresholdAuth` en los **cinco** circuitos que lo consumen
(`mint`, `mint_to_pending`, `freeze`, `recovery`, `governance`). Eso ultimo
es cirugia en la creacion de dinero y va con la misma cautela que §50.

## 53. Entrada 37: el barrido de disposiciones, y la herramienta que lo hace

§50.7 dejo abierta la pregunta incomoda: el solapamiento de §38 produjo
**tres** fallos de solidez en los tres constructores de compromisos, y
**nadie habia mirado los otros once circuitos**. La 37 la cierra, y no con
una lectura sino con una herramienta: `tools/check_constraint_layout.py`.

### 53.1 Que detecta, y por que por indice absoluto

Para cada circuito resuelve la cadena de constantes a numeros, expande cada
`result[...]` sobre los bucles que lo envuelven, y cruza los conjuntos de
**indices absolutos** buscando tres cosas:

- **COLISION** — una ranura escrita por dos sitios distintos del codigo. Es
  la firma exacta de §38: el segundo pisa al primero y la restriccion pisada
  no se impone.
- **DESBORDE** — una ranura por encima de `NUM_CONSTRAINTS`.
- **MUERTA** — una ranura declarada que nadie escribe.

⚠️ **Por indice absoluto, no por grupo**, y esto importa: en los circuitos
de dos carriles `result[C_HASH_A + lane * 12 + i]` escribe a proposito dentro
del rango de `C_HASH_B` cuando `lane = 1`. Comprobar «cuantas ranuras usa cada
grupo» daria un falso positivo ahi. Lo que define el defecto no es invadir un
rango ajeno: es que **dos sentencias distintas** escriban la misma ranura.

### 53.2 Resultado del barrido

**14 circuitos, ninguna colision, ningun desborde, ninguna ranura muerta.**

Los tres fallos conocidos estan corregidos y **no hay un cuarto** de esta
clase en los once circuitos que nadie habia contado. La pregunta de §50.7
queda respondida.

### 53.3 La herramienta esta probada contra el fallo real

Un detector que nunca ha detectado nada no esta probado. `--autotest`
reproduce la disposicion que tenia `circuit_send` la manana del 30-07-2026
—`C_TRANSPORT` declarando 7 ranuras y escribiendo 15— y comprueba que
**caza las 8 ranuras pisadas**, que fueron el fallo de §50. Se ejecuta sola:

```
python3 tools/check_constraint_layout.py --autotest
python3 tools/check_constraint_layout.py
```

### 53.4 Tres errores mios construyendo la herramienta

**Uno, de diseño.** La primera version comprobaba «cuantas ranuras usa cada
grupo frente a las que declara». Habria dado falsos positivos en todos los
circuitos de dos carriles, por lo de §53.1. El diseño correcto —colision entre
sentencias— solo aparecio al mirar por que el primer resultado no cuadraba.

**Dos, de uno-en-uno.** El contador de elementos de un array sumaba comas + 1,
lo que da uno de mas cuando hay **coma final** —que es el estilo del proyecto—.
Resultado: la primera ejecucion reporto **nueve colisiones inexistentes**,
incluidas dos en `circuit_send` y `circuit_claim`, los circuitos que se
acababan de corregir. Estuve a punto de anunciar que el arreglo de la manana
estaba incompleto. Lo desmintio mirar el array: tenia siete elementos, no
ocho.

**Tres, en el propio autotest.** Espere 8 colisiones en un caso sintetico que
solo reproducia la mitad del fallo (la identidad, sin el aleatorio) y por
tanto daba 4. Copie el numero del circuito real sin ajustarlo a mi version
simplificada.

> Los tres son el mismo error de siempre, ahora sobre la herramienta en vez
> de sobre el circuito: **operar sobre lo que uno cree que hay**. Y los tres
> los cazo lo mismo: contrastar la salida con el codigo antes de creersela.
> Una herramienta de auditoria sin verificar es una afirmacion sin verificar.

### 53.5 Lo que este barrido NO cubre

Detecta que **cada ranura se escriba exactamente una vez**. No dice nada
sobre si lo que se escribe en ella es **correcto**: una restriccion bien
colocada pero mal formulada pasa el barrido sin problema. §39 fue justo eso
—una restriccion ausente, no pisada— y esta herramienta **no lo habria
encontrado**. Para esa clase sigue sin haber mas instrumento que la lectura
semantica y el test discriminante, que es el argumento de §40.4 y de la
entrada 7.

## 54. El umbral, reconstruido fuera del circuito — y lo que falta

Al separar los carriles (§52.7) el umbral 2-de-N dejo de vivir en el
circuito: el orden estricto `idx_b - idx_a - 1` que garantizaba «custodios
distintos» desaparece con la traza conjunta. `verify_threshold_pair`
—anadida el 30-07-2026— es donde vive ahora.

### 54.1 Las tres comprobaciones, y por que en ese orden

1. **Las dos raices son la que dice la capa.** No se leen de las pruebas.
2. **Los nulificadores difieren.** Aqui el umbral es umbral.
3. **Las dos pruebas verifican.**

### 54.2 Un ataque que no estaba en el analisis previo

§51 y §52 identificaron **un** problema al separar: reimponer «custodios
distintos». Al escribir la funcion aparecio un **segundo**, que ninguno de
los dos analisis habia visto:

> **El atacante trae su propio conjunto de custodios.** Se construye un arbol
> con dieciseis claves suyas y firma dos veces con dos de ellas. Las dos
> pruebas son **validas**. Los nulificadores son **distintos**. La
> comprobacion de «custodios diferentes» pasa sin problema.

Lo unico que lo detiene es que la raiz del conjunto **la ponga la capa** y no
salga de la prueba. En el circuito conjunto este agujero no existia: habia
una sola prueba con una sola raiz. **Lo abre la separacion**, igual que el
del orden estricto. Dos agujeros, no uno, y el segundo solo aparecio al bajar
a escribir la funcion —no leyendo el diseño—.

### 54.3 Un acierto de diseño que conviene nombrar

`PairRejection` es un enum, y los tests comprueban `Err(SameCustodian)` o
`Err(WrongCustodianSet)`, no un booleano. Es decir: la disciplina de §16.5
—que un test negativo rechace **por la razon correcta**— queda impuesta por
el **tipo**, no confiada a un comentario. Los tests de §39, §50 y §50.7 tuvieron
que comprobarlo aparte porque devolvian `bool`.

### 54.4 ⚠️ Lo que falta, y es grave: la autorizacion no dice QUE autoriza

`verify_threshold_pair` demuestra que **dos custodios distintos del conjunto
autorizaron ALGO**. Las entradas publicas son la raiz y el nulificador: no
mencionan destinatario, importe ni contador.

> **Un par valido se puede reproducir.** Dos custodios autorizan emitir 1.000
> a Alicia; cualquiera reenvia esas mismas dos pruebas para emitir 1.000.000
> a Bob, y verifican igual.

Es exactamente el hallazgo de §41.2 —la autorizacion es posesion de claves,
no aprobacion de una operacion— ahora **concentrado en un solo punto** en vez
de repartido por cinco circuitos. Cerrarlo es atar un identificador de
operacion al nulificador, `H(dominio, clave, operacion)`, y llevarlo a las
entradas publicas. Eso ademas **cierra la enlazabilidad** de §52.4: el
nulificador dejaria de ser estable entre operaciones. Una sola pieza resuelve
las dos cosas, y esta sin hacer.

⚠️ **Mientras tanto, este camino NO es utilizable en produccion**, y la
funcion lo dice en su propia documentacion. Lo que hay es un experimento
completo y medido, no un sustituto de `ThresholdAuth`.

## 55. La autorizacion cubre la operacion: una pieza, dos agujeros cerrados

§54.4 dejo el hueco grave: `verify_threshold_pair` demostraba que dos
custodios distintos autorizaron **algo**, no **esta** operacion, asi que un
par valido se reproducia. Y §52.4 dejo otro: el nulificador era estable, luego
enlazable entre operaciones. **Los dos los cierra la misma pieza.**

### 55.1 La atadura

```text
nulificador = H(NULLIFIER_DOMAIN, clave, operacion)
```

Y encaja **sin coste en filas**: `native_merge` absorbe ocho elementos por
permutacion, asi que dominio y clave van en la mitad izquierda y el
compromiso de la operacion en la derecha. Las mismas 8 filas de antes; solo
crece la traza en 4 columnas (14 -> 18) y las restricciones (35 -> 39).

- ✅ **Sin reproduccion**: las entradas publicas nombran la operacion, y
  `verify_threshold_pair` recibe la que la capa ejecuta y las contrasta
  (`WrongOperation`).
- ✅ **Sin enlazabilidad**: el nulificador cambia con cada operacion.
- ✅ **Conserva lo necesario**: dentro de UNA operacion sigue siendo estable,
  que es lo que permite exigir custodios distintos. Romper esto habria
  devuelto el umbral 2-de-N a 1-de-N por la puerta de atras.

### 55.2 La cadena que lo sostiene, y el test que la prueba

La atadura vale lo que valga su eslabon mas debil:

> asercion en la fila 0 (la operacion declarada es la de la traza)
> -> **constancia de `COL_OP` entre filas**
> -> el hash del nulificador lee `COL_OP` en la fila 39.

Si la constancia estuviera muerta —como lo estaba la de `COL_SALT` en §50.7—
un custodio pondria la operacion declarada en la fila 0 y otra en la 39,
obteniendo **nulificadores distintos para si mismo**. El umbral caeria a
1-de-N sin que nada lo delatara.

El barrido de §53 dice que la ranura esta bien asignada. **Eso no basta**, y
§53.5 ya lo advertia: el barrido comprueba la disposicion, no la correccion.
`an_operation_inconsistent_across_the_trace_is_rejected` construye el testigo
malicioso y comprueba que **no cuela**.

### 55.3 Medicion actualizada

El circuito cambio, asi que la tabla de §52.1 esta rancia. Medido de nuevo:

| circuito | cols | filas | restr. | ms | bytes |
|---|---|---|---|---|---|
| A single (indice publico, descartada) | 16 | 64 | 26 | 2,9 | 14.700 |
| **B single (con operacion atada)** | 18 | 64 | **39** | 3,3 | 16.215 |
| conjunto (dos carriles) | 34 | 64 | 60 | 4,4 | 20.122 |

**Dos pruebas B: 6,6 ms y 32.430 bytes. Una conjunta: 4,4 ms y 20.122 bytes.**
Factor 1,5× en tiempo y 1,6× en tamano, sobre operaciones raras. **El veredicto
de §52 se mantiene: la via B es viable**, ahora con la operacion atada.

⚠️ Los tiempos absolutos bajaron respecto a §52.1 (2,9 frente a 4,1 ms en la
variante A, que no cambio). Es varianza de la maquina, no una mejora: lo
comparable son las **proporciones**, no los milisegundos.

### 55.4 Una cifra rancia mia, y como se ha cerrado la clase

La tabla decia **35** restricciones para B cuando ya eran **39**: la escribi a
mano al montar la medicion y no se actualizo al atar la operacion. Es la
cuarta cifra fija que falla en esta sesion, tras el «29» que eran 60 (§52.6) y
las dos que retire por error (§42.5, §48.3).

Se ha corregido **de raiz**: `NUM_CONSTRAINTS` es ahora `pub` en los dos
circuitos y la tabla **la lee del codigo**. Una tabla de medicion con numeros
escritos a mano es una cifra sin fuente esperando su turno.

### 55.5 Que queda de la 33

Una sola cosa: **sustituir `ThresholdAuth` en los cinco circuitos** que lo
consumen (`mint`, `mint_to_pending`, `freeze`, `recovery`, `governance`).
Todo lo demas —diseño, medicion, decision de variante, umbral reconstruido
fuera del circuito, operacion atada— esta hecho y verificado con 18 tests.

Esa sustitucion es cirugia en la creacion de dinero y va con la cautela de
§50: test discriminante antes de tocar nada.

## 56. El puente con los cinco circuitos, y por que la cirugia no se hizo hoy

De la 33 quedaba «sustituir `ThresholdAuth` en los cinco circuitos». Al bajar
al codigo, esa frase escondia dos cosas: una **amputacion** mayor de lo que
sugeria, y una **pieza que no existia**.

### 56.1 Que significa de verdad la sustitucion

`circuit_mint` **empotra la subida de custodios dentro de su propia traza**:
filas 272-311, columnas 26-31 y 43-44, con sus restricciones y sus grados.
Los otros cuatro hacen lo mismo. Sustituir no es cambiar una firma: es
**amputar** ese tramo de cada circuito —que queda mas pequeño— y que la
autorizacion pase a ser una prueba aparte que la capa exige junto a la de la
operacion.

### 56.2 La pieza que faltaba: atar la autorizacion a ESTA operacion

Si `mint` deja de verificar la autorizacion internamente, **nada obliga a que
la autorizacion corresponda a esa emision**. Una autorizacion valida para
emitir 1.000 serviria para emitir 1.000.000: es el agujero de §54.4 otra vez,
ahora **entre circuitos** en vez de entre operaciones.

`commit_operation(dominio, parametros)` lo cierra. Resume que se autoriza en
un `Digest` que los custodios firman; la capa lo calcula desde las entradas
publicas de la operacion y exige que coincida.

**Un dominio por tipo de operacion** (`OP_MINT`, `OP_MINT_PENDING`,
`OP_FREEZE`, `OP_RECOVERY`, `OP_GOVERNANCE`): sin eso, los mismos parametros
bajo otro significado colisionarian y una autorizacion de congelacion valdria
como autorizacion de emision.

⚠️ **Y una suposicion declarada en la propia funcion**: la esponja **no lleva
relleno**, asi que supone longitud FIJA por dominio. Hoy se cumple —cada
operacion tiene un numero fijo de parametros— y los dominios impiden
colisiones entre tipos. **Si alguna operacion pasa a tener parametros de
longitud variable, esto necesita una regla de relleno antes de usarse.** Queda
escrito porque es la clase de suposicion que se olvida y muerde anos despues.

### 56.3 Lo que se ha construido y probado

El puente, no la cirugia. Cuatro tests nuevos, entre ellos el que cierra el
diseño: dos custodios autorizan emitir 1.000, alguien intenta usar esas
autorizaciones para emitir 1.000.000, y el par se rechaza con
`WrongOperation`. **16 tests** en el modulo de la variante B.

### 56.4 Por que la cirugia no se hizo, con criterio y no por cansancio

Amputar el tramo de custodios de cinco circuitos de creacion de dinero, sin
compilar entre paso y paso, al final de una sesion de este tamaño, es
exactamente el acto contra el que esta auditoria ha trabajado todo el dia.
§50 y §50.7 fueron dos fallos de solidez que vivieron meses en circuitos que
parecian correctos.

**El camino correcto es circuito a circuito**, empezando por `mint`, con test
discriminante antes y despues: comprobar que el circuito amputado **rechaza**
una emision cuya autorizacion no corresponde, que es la propiedad que la
amputacion pone en riesgo. Eso es una sesion con toolchain por circuito, no
un parche.

Lo que hoy queda hecho es que esa cirugia **ya tiene su andamio**: el
mecanismo de autorizacion existe, esta medido, tiene la operacion atada, y
ahora tambien el compromiso que la enlaza con cada circuito.

## 57. El piloto de la sustitucion: gobernanza, sin claves

§56.4 dejo la cirugia para «circuito a circuito». Se empezo por `mint` y el
codigo redirigio el plan.

### 57.1 Por que `mint` no era el piloto

AVISO: **lo que sigue resulto ser falso**, y se corrige en 64.5. La
maquinaria de custodios de `circuit_mint` SI es separable: ocupa las filas
272-311, un tramo propio, exactamente igual que en `circuit_recovery` -que se
extrajo sin problema (64)-. La razon real para dejar `mint` al final es su
tamaño y que crea dinero, no que este entrelazado.

~~En `circuit_mint` la maquinaria de custodios **no es separable**:~~
`C_HASH_A/B`, `C_CAP_A/B` y `C_PLACE_A/B` sirven **a la vez** a la subida al
arbol de cuentas (filas 0-271) y a la de custodios (272-311) —los mismos dos
carriles, distintos tramos—. Amputar ahi es separar tejido compartido en el
circuito mas grande de los cinco (118 ranuras, 45 columnas, 512 filas).

`circuit_governance`, en cambio, prueba **exactamente dos cosas**: que dos
miembros distintos del conjunto de gobernanza autorizaron —toda su
maquinaria— y que `count_new = count_old + 1` —**una** restriccion—.
Amputarle los custodios no deja un circuito mas pequeño: **no deja circuito**.
Lo que queda es una suma que la capa hace en Rust.

Eso convierte gobernanza de «cirugia» en «reemplazo limpio», y la hace el
piloto correcto: demuestra el patron entero con el menor riesgo.

### 57.2 ⚠️ El fallo de diseño que los tests encontraron

`§47`, `§51`, `§52`, `§54`, `§55` y `§56` diseñaron el mecanismo de
autorizacion. **Ninguna vio esto**: el circuito llevaba `CUSTODIAN_DOMAIN`
**incrustado**, y el proyecto usa **dos dominios distintos a proposito**
—`CUSTODIAN_DOMAIN` y `GOVERNANCE_DOMAIN`— porque es lo que separa a quien
puede emitir de quien puede cambiar quien emite.

El piloto asumio que un circuito servia para los dos conjuntos. **Es falso**,
y solo se vio al intentar usarlo: los cuatro tests fallaron, dos de ellos
rechazando por «conjunto equivocado» cuando deberian haber pasado.

Seis secciones de diseño sobre papel, y el fallo aparece a la primera
ejecucion. Es el patron de toda la sesion, ahora sobre mi propio diseño.

**Arreglo**: el dominio pasa a ser parametro y **entrada publica**, y
`verify_threshold_pair` lo comprueba con su propio error
(`WrongIdentityDomain`). La raiz ya lo implicaba —otro dominio da otras hojas
y otra raiz— pero comprobarlo aparte convierte un «conjunto equivocado»
opaco en «esto es una autorizacion de custodios y aqui se exige una de
gobernanza». **La jerarquia queda explicita en vez de implicita.**

### 57.3 Lo que hace `apply_governance_delegated`

Recibe dos pruebas de `circuit_threshold_single_nullifier` generadas **en las
maquinas de los miembros**, calcula el compromiso de la operacion desde su
propio estado —raiz vieja, raiz nueva, contador— y exige que las dos
autorizaciones lo lleven. Despues aplica el cambio y comprueba el contador en
Rust.

⚠️ **La garantia se muda del circuito al codigo, y hay que decirlo.** Con
`apply_governance`, si alguien intentara cambiar el conjunto de custodios sin
autorizacion, **el circuito lo impediria**. Con la via delegada, lo unico que
lo impide es que esta funcion llame a `verify_threshold_pair`. Es una
responsabilidad que estaba en matematicas y pasa a estar en Rust —mas fragil
por naturaleza—, y por eso lleva cuatro tests de rechazo:

| test | que protege |
|---|---|
| cambio legitimo se aplica | que los negativos no pasen por la razon equivocada |
| mismo miembro dos veces | que 2-de-N no sea 1-de-N |
| autorizacion para otra raiz | la atadura de §56 al nivel de la capa |
| claves de custodio | **la jerarquia**: quien emite no cambia quien emite |

Los tres negativos comprueban ademas que **el estado no cambio**: un rechazo
que ya hubiera modificado la raiz seria peor que ningun rechazo.

### 57.4 Que queda

Cuatro circuitos: `freeze` (76 ranuras), `recovery` (114), `mint` (118) y
`mint_pending` (125). El patron esta probado; `mint` va **el ultimo**, cuando
los otros tres lo hayan ejercitado, porque es donde el tejido esta mas
entrelazado.

Y las dos vias conviven: `apply_governance` sigue ahi. Retirarla es una
decision aparte, y no se toma hasta que la delegada haya sido ejercitada.

## 58. Los cuatro que quedan no son repeticion mecanica

§57 cerro el piloto en `governance` y dejo escrito «falta repetir el patron».
Al ir a `freeze`, esa frase resulto inexacta y conviene corregirla antes de
que oriente mal a nadie.

### 58.1 Gobernanza era un caso especial

`circuit_governance` era **casi solo autorizacion**: al amputarla no quedaba
circuito, solo `count_new = count_old + 1`, que la capa hace en Rust. Por eso
el piloto salio limpio.

Los cuatro que quedan **tienen contenido propio que debe sobrevivir**:

| circuito | lo que prueba ademas de la autorizacion |
|---|---|
| `freeze` | transicion del arbol de congelados (filas 0-191) |
| `recovery` | cambio de clave de una cuenta |
| `mint` | movimiento de saldo y suministro, con tope |
| `mint_pending` | lo anterior mas el compromiso del pendiente |

Y en los cuatro los carriles de hash (`C_HASH_A/B`, `C_CAP_A/B`) estan
**compartidos** entre su subida propia y la de custodios. Amputar es separar
tejido, no extirpar un bloque.

### 58.2 `dual_climb` no sirve tal cual

`freeze` prueba, en su parte propia, exactamente lo que `dual_climb`: subida
dual con hermanos compartidos, o sea «una posicion del arbol cambio».
Reutilizarlo habria evitado escribir circuito. **Pero `dual_climb` opera a
profundidad `TREE_DEPTH` = 32 y el arbol de congelados tiene
`FROZEN_DEPTH` = 24.** No encaja sin parametrizar la profundidad, que es un
cambio en una pieza que hoy funciona.

### 58.3 ⚠️ Y de paso: `circuit_freeze` prueba menos de lo que su nombre dice

`FROZEN_MARK` **no aparece en ninguna restriccion ni en ninguna asercion** de
`circuit_freeze`. Las hojas del arbol de congelados son valores **libres**: el
circuito prueba que dos hojas suben a las dos raices con los mismos hermanos,
no que se haya escrito una marca de congelado.

**No es un fallo de solidez**, y conviene decir por que:

- Los custodios autorizan las **raices**, no las hojas. Lo que firman es la
  transicion concreta.
- Dejar las hojas libres es lo que permite **congelar y descongelar con el
  mismo circuito**.
- Y es lo que mantiene **privada la identidad**: «se sabe que alguien fue
  congelado, no quien», como declaran sus propias entradas publicas.

Pero un lector supondria que el circuito ata la marca, y **no la ata**. Queda
escrito: lo que garantiza que la congelacion sea una congelacion es que dos
custodios firmaron esa transicion de raiz, no una restriccion aritmetica.

### 58.4 Lo que esto cambia en la planificacion

Cada uno de los cuatro es **una sesion con toolchain**, no un parche:
escribir el circuito amputado —o parametrizar `dual_climb` para `freeze`—,
la via delegada en la capa, y sus tests de rechazo. El orden por tamaño:
`freeze` (76 ranuras), `recovery` (114), `mint` (118), `mint_pending` (125),
con `mint` y `mint_pending` al final por tener el tejido mas entrelazado.

Lo que §57 dejo probado sigue en pie: **el patron funciona**. Lo que §58
corrige es la estimacion de cuanto queda.

## 59. El barrido de §53 solo cubria la mitad, y no lo decia

Al empezar `freeze` (entrada 33) aparecio un fallo en la herramienta de §53 y
un error mio al registrarla. Va primero porque afecta a una entrada ya
cerrada.

### 59.1 El circuito amputado de `freeze`

Las filas 0-191 de `circuit_freeze` son **autonomas**: la subida dual al arbol
de congelados, carril A el estado antes, carril B el despues, con los hermanos
compartidos. Las 192-231 son la autorizacion, reutilizando los mismos
carriles. Amputar deja exactamente `dual_climb` a profundidad
`FROZEN_DEPTH` = 24.

`circuit_frozen_climb.rs` es eso: **una copia de `dual_climb` con la
profundidad cambiada**. Se prefiere copiar a parametrizar la profundidad con
genericos constantes, porque eso ultimo toca un circuito que hoy funciona y no
conviene hacerlo en el mismo paso que la amputacion. ⚠️ **Queda como deuda
declarada**: si uno de los dos se corrige, el otro necesita la misma
correccion. Esta escrito en la cabecera del fichero.

### 59.2 ⚠️ El fallo: la herramienta saltaba DIEZ circuitos en silencio

`check_constraint_layout.py` solo entendia escrituras de la forma
`result[C_ALGO + i]`. Los circuitos que indexan con **numeros crudos**
—`result[24 + i]`, `result[44]`, como hace `dual_climb`— no se analizaban, y
la herramienta **no lo decia**: su resumen anunciaba «14 circuitos: ninguna
ranura colisiona», que un lector toma por «todos».

Eran **10 de 24**, y no son piezas menores: `compliance_circuit` (25
escrituras), `solvency` (12), `nullifier_tree` (10), `dual_climb`, `merkle`,
`nullifier`, `range_check`, `rescue_hash`, `lib`.

**Y §53.2 registro esa cobertura parcial como si fuera total**: *«no hay un
cuarto de esta clase en los once circuitos que nadie habia contado»*. Esa
afirmacion cubria catorce ficheros, no todos. La entrada 37 se cerro sobre
ella.

> Es la falsa seguridad exacta contra la que avisa la documentacion de la
> propia herramienta: *«un barrido que aprueba lo que no entiende es peor que
> no tener barrido»*. Lo escribi al construirla y aun asi lo cometi, porque
> la herramienta callaba lo que saltaba en vez de declararlo.

### 59.3 Corregido, y el resultado

Dos arreglos:

1. **El indice se evalua entero**, no como «constante mas desplazamiento».
   Ahora entiende `result[24 + i]` y `result[lane * STATE_WIDTH + i]`.
2. **Los comentarios se ignoran.** `mutation.rs` ilustra las restricciones
   vacuas escribiendo `result[C_X]` en su prosa, y el barrido lo tomaba por
   codigo. Un aviso falso gasta la atencion que hace falta para los
   verdaderos.

**Resultado: 24 circuitos, ninguna colision, ningun desborde, ninguna ranura
muerta.** La conclusion de §53.2 era **correcta**, pero no estaba
**verificada**. Ahora lo esta.

⚠️ Se deja anotado que lo estuvo por suerte durante ocho commits: los diez
circuitos sin comprobar podrian haber tenido un cuarto §50 y el repositorio
habria afirmado lo contrario.

### 59.4 Lo que sigue faltando en `freeze`

El circuito amputado existe pero **no tiene tests propios ni via delegada en
la capa**. Falta: comprobar que compila y prueba, escribir
`apply_freeze_delegated` con el compromiso `OP_FREEZE`, y sus tests de
rechazo —como los cuatro de §57.3—.

## 60. `freeze` delegado: el patron con circuito que sobrevive

Segunda aplicacion del patron de la 33, y la primera en una operacion que
**conserva contenido propio**. Gobernanza no dejaba circuito al amputarla
(§57.1); `freeze` si.

### 60.1 Las tres piezas

- `circuit_frozen_climb`: la subida dual al arbol de congelados, extraida de
  las filas 0-191 de `circuit_freeze`. Cinco tests.
- `apply_freeze_delegated`: recibe la prueba del arbol y **dos de custodios
  distintos**, generadas en sus maquinas.
- El compromiso `OP_FREEZE` sobre `[raiz_vieja, raiz_nueva, contador]`.

### 60.2 La longitud de traza tenia que ser potencia de dos

`24 x 8 = 192` no lo es, y winterfell lo exige. Se sube a 256 con ocho
niveles de **relleno** que siguen subiendo con hermano cero, y las raices se
anclan en `ROW_ROOT = 191`.

Rellenar con ceros NO funciona: la fila 191 es de enlace, y con el estado
siguiente a cero la restriccion de colocacion falla. Hay que seguir subiendo.

Y explica una decision del proyecto que llevaba a la vista sin preguntarse:
**por que `circuit_freeze` usa 512 filas** para una subida de 24 niveles mas
una de 4. No era holgura: era la potencia de dos siguiente.

### 60.3 Para que sirve la prueba del arbol si la capa recalcula

`apply_freeze` ya aplica el cambio sobre una copia y comprueba la raiz, asi
que **para la capa la prueba STARK es redundante**. Su valor es para **quien
audita el registro desde fuera**: sin ella el log guardaria una transicion de
raiz que nadie mas puede comprobar. Queda escrito en la funcion, porque un
lector podria quitarla creyendo que sobra.

### 60.4 La jerarquia, cerrada en las dos direcciones

§57 comprobo que los custodios no pueden cambiar quien custodia. §60 anade la
inversa: **gobernanza no puede congelar**. Puede cambiar quien ejerce la
custodia, no ejercerla. Que las dos direcciones esten cerradas es lo que hace
real la separacion, y ahora las dos tienen test.

### 60.5 Que queda

`recovery` (114 ranuras), `mint` (118) y `mint_pending` (125). El patron
lleva dos aplicaciones y ha necesitado un ajuste distinto en cada una: en
gobernanza el dominio de identidad (§57.2), en freeze la potencia de dos.
No conviene darlo por mecanico.

## 61. Limpiar avisos del compilador, y el primer commit roto de la sesion

`cargo build --tests` acumulaba **diez avisos** en `zk-ssl` que nadie leia.
Merecen mirarse uno a uno y no silenciarse: dos decian `unused variable:
bob`, y una cuenta de prueba preparada y no usada **podria significar que el
test no comprueba lo que su nombre dice**.

No era el caso. Los dos tests obtienen la identidad esperada **por otro
canal** -del propio destinatario- que es justo la propiedad que comprueban,
asi que el indice de la cuenta sobra. Ahora esta escrito en el codigo, para
que el proximo que vea `_bob` sepa que es deliberado en vez de encontrarse un
guion bajo mudo.

### 61.1 Dos errores encadenados

**Uno: un reemplazo por patron caso donde no debia.** Se busco una secuencia
de tres lineas creyendo que identificaba un test concreto, y esa secuencia
aparece en varios. Se renombro `bob` en un test de la linea 786 que **si lo
usa** (`let _ = bob;` veinte lineas mas abajo).

**Dos, y peor: se empujo sin verificar.** El `cargo test` no imprimio ninguna
linea `test result` -porque no llego a compilar- y el commit se dio por bueno
igual. **La ausencia de una salida esperada es informacion**, y se trato como
ruido. `c97ae47` dejo `main` sin compilar unos minutos; `80ae6a1` lo corrige y
lo nombra.

### 61.2 Donde se bajo la guardia

Es el **primer commit roto en toda la sesion**, despues de decenas de cambios
mucho mas delicados -tres fallos de solidez, cirugia en circuitos de creacion
de dinero- todos verificados antes de empujar.

Y ocurrio limpiando avisos del compilador: la tarea que parecia no poder
fallar. La disciplina se relajo exactamente donde el riesgo se creia nulo.

> Un aviso del compilador es auditoria gratuita. Ignorarlos durante diez
> commits fue una perdida; arreglarlos sin verificar fue peor.

## 62. Dos herramientas, y ninguna cubre lo de la otra

Al ir a `recovery` (entrada 33) aparecio que el proyecto tiene **una segunda
herramienta de auditoria de circuitos** que no se habia nombrado en esta
sesion: `crate::mutation::buscar_vacias`, prueba por mutacion que perturba la
traza celda a celda y comprueba que **cada** restriccion reacciona a algo.

### 62.1 Cubren defectos distintos, y conviene decirlo

| herramienta | detecta | NO detecta |
|---|---|---|
| `buscar_vacias` | restricciones que no imponen nada | el solapamiento de §38: la ranura sobrescrita **si** reacciona, solo que a la restriccion equivocada |
| `check_constraint_layout.py` | ranuras escritas dos veces, desbordadas o muertas | si lo escrito impone algo |

Ninguna es redundante y ninguna basta. Que cada una **declare lo que no
cubre** es lo unico que evita repetir la falsa seguridad de §59.2.

### 62.2 Cobertura: 15 de 24, y tres eran deuda mia

Doce circuitos tenian el test de vacuidad. **Los tres que se añadieron hoy
-`circuit_threshold_single`, `circuit_threshold_single_nullifier` y
`circuit_frozen_climb`- no lo tenian**: deuda creada en esta misma sesion.
Añadido; los tres pasan.

Quedan **nueve sin cubrir**: `compliance_circuit`, `double_entry`,
`dual_climb`, `lib`, `merkle`, `nullifier`, `range_check`, `rescue_hash` y
`solvency`. Se abre como entrada 38.

⚠️ Es el mismo patron que §59.2: una herramienta util que se aplica a parte
del codigo, sin que en ningun sitio conste a que parte. La diferencia es que
esta vez se ha contado antes de afirmar nada.

## 63. Entrada 38 cerrada: 23 de 23 sin restricciones vacuas

Se añadio `no_constraint_is_vacuous` a los ocho circuitos que quedaban
-`dual_climb`, `merkle`, `nullifier`, `range_check`, `rescue_hash`,
`solvency`, `compliance_circuit` y `double_entry`- y **los 23 pasan**.

Ningun circuito del proyecto tiene una restriccion declarada que no imponga
nada. Y ahora esta **comprobado**, no supuesto: doce lo estaban, once no.

### 63.1 Donde se esperaba encontrar algo

`compliance_circuit` (25 escrituras de restricciones) y `double_entry` (41
columnas) eran los candidatos: la mayor superficie del proyecto sin haber
pasado nunca por esta herramienta. Estan limpios.

Es un resultado negativo y vale igual que uno positivo: la diferencia entre
«creemos que no hay restricciones vacuas» y «no las hay en los 23».

### 63.2 Lo unico que queda fuera, y por que

El `WorkAir` de `lib.rs` -el circuito de demostracion de winterfell- no se
cubre: no protege nada del sistema. Contarlo habria inflado la cobertura sin
añadir garantia.

### 63.3 Dos inconsistencias de nombres, a la vista

Al escribir los tests salieron dos cosas que nadie habia notado:

- `range_check` llama **`TRACE_ROWS`** a lo que los otros 23 llaman
  `TRACE_LENGTH`. Unico caso en el proyecto.
- `nullifier` y `rescue_hash` **no tienen `TRACE_WIDTH`**: su traza es
  exactamente el estado del hash, asi que usan `STATE_WIDTH`. Coherente
  -son los dos circuitos mas primitivos- pero conviene saberlo antes de
  escribir codigo que los toque.

Ninguna es un defecto. Quedan anotadas porque un barrido automatico que
suponga nombres uniformes fallaria en las dos, y **este proyecto ya tiene
dos casos de herramientas que saltaban ficheros en silencio** (§59.2, §62.2).

## 64. `recovery` delegado: la extraccion mas grande

Tercera aplicacion del patron, y la que mas codigo movio. `governance` no
dejaba circuito al amputarla (57.1); `freeze` dejaba una subida dual que era
`dual_climb` con otra profundidad (60); `recovery` deja un circuito con
contenido propio que **nadie mas puede recomputar**.

### 64.1 Que sale y que se queda

De 114 restricciones a **92**, de 46 columnas a **39**. Fuera: claves e
indices de custodio, sus acumuladores, la colocacion en su arbol, tres de los
cuatro segmentos de rango y la mitad del transporte.

Dentro, y es la razon de que este circuito exista: **los dos carriles
construyen su hoja con la MISMA columna `COL_BAL`**, y el segmento
superviviente la descompone en 64 bits. Una recuperacion reasigna el control,
no mueve dinero.

A diferencia de `circuit_frozen_climb` -donde las hojas libres son correctas
(58.3)- aqui no lo serian: con hojas libres, dos custodios podrian vaciar una
cuenta bajo apariencia de recuperacion, y un auditor externo solo veria dos
raices cambiando.

### 64.2 Dos cosas que aparecieron al extraer

**La fila 271 no es de enlace.** El relleno hasta la potencia de dos salio
gratis, sin los niveles ficticios que necesito `freeze` (60.2). Se vio
contando `acct_link`: cubre las filas 15 y 23..263, treinta y dos niveles, y
la 271 queda fuera.

**`COL_BIT_B` era peso muerto.** En la subida al arbol de cuentas los dos
carriles **comparten camino** -misma posicion, distinta identidad- asi que un
solo bit de direccion basta. El segundo existia solo para la subida de
custodios; al amputarla quedaba una columna siempre a cero con una
restriccion booleana encima que no restringia nada.

Lo delato un aviso del compilador -`bit_b` sin usar- que estuvo a punto de
silenciarse. Es el segundo aviso del dia que señala algo real: el primero
(`bob` sin usar) resulto correcto y merecia una explicacion; este era
verdadero peso muerto. La diferencia solo se ve mirandolos.

### 64.3 La vía delegada, y lo que la distingue

`apply_recovery_delegated` recibe la prueba del circuito y dos de custodios
distintos. Cuatro tests de rechazo, y uno de ellos es especifico de esta
operacion: **una autorizacion para recuperar hacia una identidad no sirve
para entregar la cuenta a otra**. El compromiso de operacion protege aqui lo
que mas importa: a quien se le da el control.

### 64.5 Correccion de 57.1: `mint` tampoco esta entrelazado

Al mirar `circuit_mint` para planificar, resulta que su reparto de filas es
**identico** al de `recovery`: hoja en 0-15, arbol de cuentas hasta 271,
custodios en 272-311. Y su cadena de restricciones es la misma salvo lo
propio de emitir.

57.1 dijo que en `mint` la maquinaria de custodios "no es separable" y que
amputar seria "separar tejido compartido". **Es falso.** Los dos tramos usan
los mismos carriles pero en **filas distintas**, que es justo lo que hizo
mecanica la extraccion de `recovery`.

Lo que si es cierto es que `mint` es mas grande (118 ranuras frente a 114) y
que **crea dinero**, asi que merece mas cuidado. Pero eso es criticidad, no
enredo, y conviene no confundirlas: una justifica ir despacio, la otra
habria justificado un rediseño.

### 64.4 Estado del patron

Tres de cinco: `governance`, `freeze`, `recovery`. Quedan `mint` (118
ranuras) y `mint_pending` (125), que 57.1 dejo para el final por tener el
tejido mas entrelazado -sus carriles de hash sirven a la vez a la subida al
arbol de cuentas y a la de custodios-.

## 65. Qué NO demuestra este documento

Que el sistema sea seguro. Demuestra que **el autor ha buscado sus
propios fallos de forma sistemática y ha encontrado algunos**, incluidos
dos al escribir estas páginas.

Es exactamente por eso que hace falta que lo mire alguien más.
