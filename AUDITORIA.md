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

## 65. Politica de las vias duplicadas

Con tres operaciones delegadas, cada una convive con su version antigua y
**en ningun sitio constaba cual es la buena**. Un lector de
`apply_governance` no podia saber que su pareja `_delegated` existe ni por
que. Se resuelve ahora, con tres pares, en vez de con cinco.

### 65.1 Donde esta el fallo, exactamente

No en las seis funciones. Las tres `apply_*` verifican un recibo y son
correctas; el problema es **de donde sale ese recibo**. Las que reciben las
claves son `update_custodians`, `set_frozen` y `recover`, y solo esas llevan
la marca. Marcar de mas habria diluido la señal.

### 65.2 Marcadas, no borradas

Hay **75 llamadas** a las vias antiguas, todas en tests, y esos tests
comprueban propiedades reales de la capa. Borrarlas se las llevaria por
delante. Ademas la via antigua **sigue siendo la unica** para `mint` y
`mint_pending`.

Asi que llevan `#[deprecated]` con el motivo escrito:

> *Exige las claves de custodio EN EL OPERADOR: es el fallo de la entrada 32.*

Quien las llame desde codigo nuevo lo vera **en su propio compilador**, no
enterrado en un documento.

### 65.3 El permiso va en los tests, no en la definicion

`#![allow(deprecated)]` esta en los tres modulos de test que las ejercitan a
proposito, con el motivo al lado. Ponerlo en la definicion habria apagado el
aviso para todo el mundo, que es justo lo contrario de lo que se busca.

### 65.4 Cuando se retiran

Cuando las cinco operaciones tengan via delegada. Entonces las antiguas y sus
circuitos conjuntos (`circuit_governance`, y las partes de custodio de
`circuit_freeze`, `circuit_recovery`, `circuit_mint`, `circuit_mint_pending`)
dejan de tener razon de ser.

### 65.5 Lo que esta marca NO es

⚠️ **Una barrera.** `#[deprecated]` es un aviso: nada impide que codigo de
produccion la llame con un `#[allow]`. La garantia de que las claves no
lleguen al operador **no la da esta marca**, la da usar la via delegada.

Y mientras `mint` y `mint_pending` no la tengan, **el fallo de la entrada 32
sigue abierto**: las dos operaciones que crean dinero siguen exigiendo las
claves. Tres de cinco no es la mitad del problema resuelto, porque las dos
que faltan son las que mas importan.

## 66. `mint` extraido: el circuito que prueba el tope

Cuarta aplicacion del patron, y la mas critica: `mint` crea dinero.

### 66.1 Que se queda dentro

De 118 restricciones a **96**, de 45 columnas a **38**. Fuera lo de siempre
mas tres de los ocho segmentos de rango. Dentro, lo que hace que este
circuito no pueda reducirse a una subida de Merkle:

- `saldo_nuevo = saldo + importe` y `suministro_nuevo = suministro + importe`
- **el margen del tope**: `tope - suministro_nuevo` en 64 bits

Sin el segundo, dos custodios podrian firmar una raiz cualquiera y un auditor
externo no sabria si se respeto el limite. Tiene test por los DOS lados:
emitir por encima se rechaza, y emitir hasta el tope **exacto** se acepta -un
circuito que rechazara tambien el borde seria mas restrictivo de lo declarado,
y eso solo se ve probando los dos-.

### 66.2 ⚠️ El fallo que costo dos rondas, y lo que revela

Se quitaron las periodicas de custodio de su construccion y de su uso, pero
**no de la cadena de constantes**: `P_CUST_LINK`, `P_POW2` y
`P_SEL_CUST_ROOT` seguian ahi, desplazando `P_SEG_LINK` tres posiciones. El
indice se salio del array y `prove` panico con *"len is 36 but the index is
36"*.

En `recovery` este paso si se hizo; en `mint` se olvido. Lo caza el test
positivo, no el negativo: los tres negativos pasaban porque **todo** fallaba.
Es exactamente la razon por la que un test positivo no es opcional aunque
parezca trivial.

**Y revela un hueco de cobertura**: `check_constraint_layout.py` comprueba
los indices de `result[...]` pero **no los de `periodic[...]`**. Aqui se noto
porque desbordo; si el desplazamiento fuera hacia abajo, la restriccion
leeria la columna periodica equivocada **sin ruido**. Se abre como entrada 39.

Es la tercera vez en la sesion que aparece el mismo patron -herramienta que
cubre parte del problema sin declarar que parte- despues de §59.2 (el barrido
saltaba 10 de 24 circuitos) y §62.2 (la prueba de vacuidad cubria 12 de 24).

### 66.3 Lo que el proyecto me evito

El comentario de la lista de grados del circuito original advertia que con
ocho segmentos el rango llenaba **exactamente** las 512 filas, volviendo
periodicas de periodo 64 las columnas del segmento. Al bajar a cinco
(5x64=320) eso deja de ser cierto y el ciclo vuelve a `TRACE_LENGTH`.

Sin ese comentario habria salido un panico de grados y una ronda larga. Es la
primera vez en la sesion que la documentacion del proyecto **evita** un error
en vez de al reves.

## 67. `mint` delegado: la que crea dinero

Cuarta via delegada, y la que mas pesa. Un operador con las claves podia
emitir; con esta via no.

### 67.1 El tope se comprueba DOS veces, y no es redundancia

- **En la capa**, porque conoce el suministro real y puede rechazar antes de
  generar nada.
- **En el circuito**, porque un auditor externo que solo ve el registro **no
  puede recomputar el suministro** y necesita que la prueba se lo garantice.

Es el reverso de §60.3: alli la prueba del arbol era redundante para la capa
y valiosa fuera; aqui las dos comprobaciones tienen destinatarios distintos y
las dos hacen falta.

### 67.2 El test que mas vale

`an_authorization_for_one_amount_does_not_mint_another`: se autoriza emitir
250.000 y se intenta emitir un millon. **Se rechaza.** El compromiso de
operacion (§56) cubre el importe, el suministro antes y despues, y el tope,
asi que una autorizacion no vale para otra cantidad.

Es la propiedad mas valiosa que este sistema puede tener, y hasta hoy no
estaba probada porque la autorizacion no existia como pieza separada.

Los cuatro tests comprueban ademas que **el suministro no cambio** tras el
rechazo. Un rechazo que ya hubiera emitido seria peor que ninguno.

### 67.3 Estado: cuatro de cinco

`governance`, `freeze`, `recovery` y `mint`. Queda `mint_pending` (125
ranuras), que añade el compromiso del pendiente sobre lo que hace `mint`.

⚠️ **Hasta que esa este, el fallo de la entrada 32 sigue abierto**: emitir
a un pendiente sigue exigiendo las claves en el operador. Cuatro de cinco no
cierra la entrada.

## 68. `mint_pending`: el analisis antes de cortar

Es la quinta y la mas distinta de todas. Se registra el analisis ANTES de
tocar nada, porque si algo se pierde es esto lo que costaria rehacer.

### 68.1 Esta montada al reves que las otras cuatro

En `mint`, `recovery` y `freeze` el ascenso de custodios va AL FINAL (filas
272-311) y lo propio al principio. En `mint_pending` va **al principio**:

| filas | que hay |
|---|---|
| 0-39 | ascenso de custodios (ciclo 0 identidades, ciclos 1-4 el arbol) |
| 39 | ⚠️ **doble funcion**: raiz de custodios Y arranque del compromiso |
| 40-47 | compromiso interno `H(identidad_receptor, aleatorio)` |
| 48-55 | el pendiente `H(interno, importe)` |
| 55 | entrada en el arbol de pendientes |
| 56-311 | niveles 1..31 |

### 68.2 Por que la amputacion es limpia igualmente

La fila 39 hace doble funcion, pero **el compromiso no depende del ascenso**:
`C_PEND_IN` lo ata a `COL_R_ID` y `COL_SALT` con su **propio selector**
(`P_PEND_IN`), no con el de la raiz de custodios. Asi que quitando el ascenso
el compromiso sigue arrancando donde arranca.

El precio: las filas 0-38 quedan **muertas**, y hay que apagar el indicador
de hash en ellas o las restricciones de Rescue se activarian sobre ceros.
No se ganan filas: `ROW_PENDING_ROOT`=311 obliga a 512 de todas formas.

### 68.3 Que sale y que se queda

**Fuera**: 10 columnas (bits, claves, indices, acumuladores y las dos del
segmento), 36 ranuras de restriccion y 8 periodicas. `NUM_SEGMENTS` pasa de
**3 a 0**: los tres segmentos eran indices de custodio y su orden, ninguno
era un rango de valor.

**Dentro**: toda la maquinaria del pendiente y **el tope**, que aqui tiene su
propio mecanismo (`COL_CBIT`/`COL_CACC`, `C_CAP_LINK`) separado de los
segmentos. Eso es lo que hace que quitar los segmentos enteros no se lleve
por delante la comprobacion del limite -a diferencia de `mint`, donde el
margen del tope ERA el segmento 4 (§66.1)-.

De 125 ranuras a **89**, de 49 columnas a **39**.

### 68.4 Lo que este analisis no dice

Si compila. Es la transformacion mas grande de las cinco y la unica con
filas muertas al principio: el indicador de hash, las aserciones de la fila 0
y la cadena de periodicas hay que rehacerlos, no solo recortarlos.

## 70. `mint_pending` amputado: la quinta, y la unica montada al reves

Ultima aplicacion del patron de la entrada 33. `circuit_mint_pending_climb`
prueba lo propio -suministro, compromiso, posicion libre y tope- sin saber
nada de custodios.

### 70.1 El analisis de §68 acerto en las cuentas y fallo en dos cosas

Recontado contra el codigo antes de tocar nada:

| | §68 dijo | Medido |
|---|---|---|
| Ranuras fuera | 36 | **36** ✅ |
| Ranuras | 125 → 89 | **125 → 89** ✅ |
| Columnas | 49 → 39 | **49 → 39** ✅ |
| Periodicas fuera | 8 | ⚠️ **9** |

~~Ocho periodicas.~~ **Son nueve.** `P_FIRST_ROW` lo leia **solo**
`C_KEY_INPUT`, que se va con la amputacion. Dejarla habria sido una
periodica que se construye y nadie lee: el peso muerto que §66.2 mando
retirar en `mint_climb` y que la entrada 39 declara que **nada comprueba**.

Es la cuarta vez que aparece el patron de una herramienta o un recuento que
cubre parte del problema sin declarar que parte (§59.2, §62.2, §66.2).

### 70.2 Las filas muertas, y por que el compromiso sobrevive

Este circuito llevaba el ascenso de custodios **al principio** (filas 0-39),
al reves que los otros cuatro. Amputado, las filas 0-38 quedan vacias y hay
que **apagar el indicador de hash y las ARK en ellas**, o las restricciones
de Rescue se activarian sobre ceros.

No se ganan filas: `ROW_PENDING_ROOT`=311 obliga a 512 igual.

El compromiso arranca donde arrancaba porque `C_PEND_IN` lo ata a `COL_R_ID`
y `COL_SALT` con **selector propio** (`P_PEND_IN`). `ROW_ROOT` se renombra a
`ROW_PEND_START`: conserva el 39 y pierde la doble funcion que tenia.

Las aserciones bajan de 38 a **12**. Se van las 26 de custodio; quedan las
cuatro constantes que `C_TRANSPORT_NEW` propaga a toda la traza y las ocho
de las dos raices. Fijar valores en filas que ninguna restriccion lee seria
ruido.

### 70.3 Compilo a la primera, y eso merece anotarse

§68.4 decia que el analisis **no dice si compila**, y era la transformacion
mayor de las cinco. Compilo sin un error y sin un aviso, y los 13 tests
pasan en release.

Es la **segunda vez** en la auditoria que el trabajo previo evita una ronda
en vez de causarla; la primera fue el comentario de grados de §66.3. Que el
analisis se escribiera **antes de cortar** —§68 se redacto sin tocar nada—
es lo que lo explica.

### 70.4 ⚠️ Lo que el carril B no ata, y que estos 13 tests NO miran

`C_PEND_IN` y `C_PEND_VAL` restringen **solo el carril A**: no hay ningun
`LANE_B` en ellas. Lo que entra al arbol en `C_PEND_ENTRY_B` es lo que el
carril B haya calculado, y en el circuito **nada lo ata** a `COL_R_ID`,
`COL_SALT` ni `COL_AMOUNT`.

Hoy lo sujeta la capa, que recomputa `pending_commitment` y compara raices.
Un auditor externo que solo vea la prueba no puede.

⚠️ **Es hipotesis de lectura, no medida.** Es de la clase de §39 y §27
—restriccion *ausente*, no colisionada— que `check_constraint_layout.py`
**no detecta por construccion** y para la que solo hay lectura semantica y
test discriminante (entrada 7). `the_pending_commitment_is_inserted`
comprueba **la traza**, no que el circuito lo obligue.

Es preexistente: el circuito original tiene la misma forma. Se abre como
**entrada 40**, sin degradarla a «cosmetico» —que es lo que se hizo tres
veces con la §36 antes de que un test la mirara.

## 71. `mint_pending` delegado: las cinco vias existen

Quinta y ultima via delegada. Ninguna de las cinco operaciones
privilegiadas necesita ya que las claves de custodio lleguen al operador.

### 71.1 ⚠️ Errata de §68: el fichero no es `pending.rs`

~~«OJO: el fichero es `crates/zk-ssl/src/pending.rs`».~~ **Es
`crates/zk-ssl/src/two_phase.rs`.** Comprobado contra el codigo:

- `pending.rs` no tiene `SovereignLayer`, ni `ThresholdAuth`, ni operacion
  de custodios alguna.
- Su `PendingTransfers` tiene **cero usos fuera de su propio fichero**: es
  el modelo que §37.6 ya declaro que **la capa no ejecuta**. El arbol real
  es `pending: SparseTree` en `lib.rs`.
- `mint_to_pending` y `apply_mint_to_pending` viven en `two_phase.rs`, y
  las cuatro delegadas anteriores van **todas** en el mismo fichero que su
  via antigua.

§68 acerto en la mitad util del aviso —no hay `mint_pending.rs`— y fallo en
el destino. Se registra porque un aviso a medias dirige mal con la
autoridad de un aviso.

### 71.2 La posicion la asigna la capa, y cierra en falso

Quien genera la prueba necesito el camino del arbol, luego ya conocia la
posicion. Si la capa asigna otra, la raiz nueva no coincide y **falla la
verificacion de la subida**.

Y el compromiso de operacion ata `raiz_vieja ++ raiz_nueva ++ importe ++
suministros ++ tope`, asi que una autorizacion no vale **ni para otro
importe ni para otro hueco**: las dos raices fijan las dos cosas. Es §67.2
ampliado.

### 71.3 El grado de la posicion 0: medido, no supuesto

Los cinco tests fallan en **depuracion** con *"transition constraint degrees
didn't match"*. Diagnostico propuesto: la posicion 0 tiene el camino de
Merkle todo a la izquierda, luego `COL_PBIT` es identicamente nula, luego
los veinte terminos `pbit * X` se anulan.

Eso era un razonamiento sobre indices, y razonar sobre indices ha fallado
tres veces en esta auditoria. Se hizo **test discriminante**: misma traza,
mismo circuito, **solo cambia el camino**.

| indices | que son | declarado | posicion 0 |
|---|---|---|---|
| 50-69 | los veinte terminos con `pbit` | 1022 | **511** |
| 70 | `C_PBIT_BOOL` | 511 | **0** |

Las otras 68 posiciones, identicas. Y el vector coincide **exactamente** con
el que emitia la capa. `the_all_left_path_of_position_zero_still_verifies`
pasa en **release**: el circuito es correcto tambien ahi.

Es §37.7 reproducido —alli, intervenir en `allocate_pending` no dejaba «ni
una `C_PEND_*` ni `C_PBIT_BOOL` desviada»— y cae bajo la decision de §46:
**se declara, no se migra**.

⚠️ **No se cambio el test a otra posicion para que pasara.** En produccion
la posicion 0 se usa —`allocate_pending` reutiliza huecos y un ledger recae
en ella (§46.1)—, asi que un test en la posicion 1 pasaria sin ejercitar el
caso comun. Vale mas un test fiel saltado que uno verde mirando a otro lado.

### 71.4 ⚠️ Y NO se marcaron los otros doce

`mint`, `freeze` y `recovery` acumulan **doce** fallos de depuracion, y se
comprobo con `git stash` que son **preexistentes**: sin este trabajo fallan
los mismos doce.

Pero divergen en los indices **44 y 73-88**, no en 50-70. Otra disposicion,
**causa no medida**. Ponerles el motivo de la posicion 0 seria atribuirles
una causa sin comprobar. Van como **entrada 41**, con el metodo que acaba de
funcionar aqui.

Los cinco de `mint_pending` si llevan
`#[cfg_attr(debug_assertions, ignore = ...)]` con el motivo medido. La
asimetria es **epistemica, no cosmetica**, y queda declarada: es la leccion
de §59.2, §62.2 y la entrada 39 —declarar hasta donde llega lo que se ha
comprobado.

### 71.5 Lo que esto cierra, y lo que no

**Cierra**: las cinco operaciones tienen via que no exige las claves.

⚠️ **No cierra la entrada 32.** Las vias antiguas siguen siendo llamables
con un `#[allow(deprecated)]`. §65.4 dice que se retiran cuando las cinco
tengan delegada —que es ahora—, y §65.5 avisa de que la garantia **no la da
la marca, la da usar la via delegada**. Marcar la 32 resuelta hoy seria
justo ese maquillaje.

## 72. Entrada 40 CONFIRMADA: el compromiso del pendiente no esta atado

⚠️ **Cuarto fallo de solidez de la auditoria.** Medido en release el
31-07-2026, en los DOS circuitos.

### 72.1 Que acepta el circuito

Una traza que **declara emitir 250.000, sube el suministro en 250.000, y
deposita un pendiente que vale 1.000.000**. `prove` no protesta y `verify`
la acepta.

El testigo se construye con dos trazas honestas -una por el importe, otra
por el cuadruple- copiando **solo el carril B** de la segunda sobre la
primera. Todo lo demas queda coherente a proposito, para que el resultado
signifique algo: la cadena de Rescue del carril B es valida porque viene de
una traza valida; los hermanos coinciden porque los fija el **camino**, no
el carril, asi que `C_PEND_SIBLING` se cumple; y las columnas constantes no
se tocan, asi que `C_TRANSPORT_NEW` y `C_SUPPLY` tambien.

### 72.2 No falta una restriccion: esta en el carril equivocado

Esto es lo que la entrada 40 no decia todavia, y afina el diagnostico.

`C_PEND_IN` y `C_PEND_VAL` construyen el compromiso sobre el **carril A**.
Pero en la fila de entrada al arbol:

```
result[C_PEND_ENTRY_A + i] = pend_entry * ((1-pbit)*next[4+i] + pbit*next[8+i]);
```

`C_PEND_ENTRY_A` fuerza la hoja del carril A a **CERO** —la posicion estaba
libre— **sin mirar el digest que el carril A trajo**. Y `C_PEND_ENTRY_B`
inserta `current[LANE_B + 4 + i]`: lo que calculo el carril B.

> **El carril A calcula durante dieciseis filas un compromiso que ningun
> lector consume, y el que se deposita es el del carril B, que nadie ata.**

Las restricciones del compromiso estan escritas sobre el carril que se
descarta. Decirlo como «falta una restriccion» habria llevado a un arreglo
que las duplica en los dos carriles; el arreglo correcto es que vivan donde
se lee el resultado.

### 72.3 Alcance: medido en el circuito, LEIDO en la capa

| | |
|---|---|
| `circuit_mint_pending_climb` | ⚠️ **Acepta.** Medido. |
| `circuit_mint_pending` (produccion) | ⚠️ **Acepta.** Medido. |
| `apply_mint_pending_delegated` | ✅ **Protegida.** Recomputa `pending_commitment` y construye `pending_root_new` ella misma; una traza inflada no casa. |
| `apply_mint_to_pending` (antigua) | ⚠️ **NO protegida, por LECTURA. Sin medir.** |

La via antigua toma `receipt.commitment` **del que llama** y solo comprueba
que cuadre con `pi.pending_root_new`, que sale de la prueba: las dos
vendrian infladas y coincidirian entre si. Si esa lectura es correcta, el
suministro sube 250.000 mientras entra al arbol un pendiente de 1.000.000
que su titular puede cobrar, y **la conservacion se rompe**.

⚠️ **Eso es lectura, no medicion**, y en este documento la diferencia ha
costado cara cuatro veces. Lo decide un test de capa, que es lo siguiente.

### 72.4 Que se necesita para explotarlo, y por que aun asi importa

Hacen falta **dos claves de custodio**. Quien las tiene ya puede emitir con
`mint`, asi que **no hay escalada de privilegio**.

Pero lo que se rompe no es el control de acceso: es el **invariante**. Una
emision por `mint` queda contabilizada en el suministro y sujeta al tope
que §66 y §67 se tomaron el trabajo de demostrar. Esta no: crea dinero
**fuera del suministro declarado y por tanto fuera del tope**, y un auditor
externo que solo vea la prueba **no puede detectarlo**.

Es el reverso exacto de §67.1: alli el tope se comprobaba dos veces porque
el auditor externo no puede recomputar el suministro. Aqui el auditor
tampoco puede comprobar que el pendiente valga lo que el suministro subio.

### 72.5 Por que ninguna herramienta del proyecto podia verlo

- `check_constraint_layout.py` cruza los indices de `result[...]`: detecta
  colisiones, desbordes y ranuras muertas. **Aqui no hay ninguna de las
  tres.** Las 89 ranuras se reparten bien; el problema es la que no existe.
- `buscar_vacias` comprueba que ninguna restriccion sea vacua. Las 89
  disparan. **Una restriccion ausente no es una restriccion vacua.**
- Los 13 tests del circuito pasan. `the_pending_commitment_is_inserted`
  comprueba **la traza**, no que el circuito la obligue —y esa distincion
  quedo dicha en §70.4 al escribirlo, antes de saber que aqui habia algo.

Es la clase de §39 y §27, y la tercera vez que aparece: **el unico
instrumento conocido para ella es la lectura semantica y el test
discriminante** (§40.4, entrada 7). Este hallazgo salio de leer el circuito
para escribir otra cosa.

### 72.6 Los testigos se quedan ROJOS

Los dos van con `#[ignore]` y su motivo, no borrados ni debilitados. Es lo
que se hizo en §50 con el testigo de `circuit_send`: rojo hasta que se
corrija, y entonces **se le quita la marca y pasa a verde**.

Un testigo borrado no vuelve. Uno debilitado miente.

⚠️ **Y conviene no confundir esto con estar arreglado.** La tanda dice
«270 passed; 2 ignored», y esos dos ignorados son un fallo de solidez
confirmado esperando correccion.

## 73. ⚠️ FALLO GRAVE: la capa no verificaba las pruebas de la via de pago

**El fallo mas grave de la auditoria**, y de una clase distinta a los tres
anteriores. §27, §30 y §36 eran restricciones mal escritas **dentro** del
circuito. Este es el circuito entero **sin conectar**.

Encontrado el 31-07-2026 leyendo `apply_mint_to_pending` para escribir el
test de capa de la entrada 40. Corregido y registrado en el mismo commit.

### 73.1 Que se midio

En `two_phase.rs` la unica llamada a `verify::<...>` era la de
`apply_mint_pending_delegated` (§71). `apply_send`, `apply_claim` y
`apply_mint_to_pending` **no verificaban su prueba**. Los demas modulos de
la capa —`burn`, `freeze`, `governance`, `audit`, `mint`, `recovery`— si.
Y `log::verify` declara en su propio comentario que no valida pruebas.

Cinco testigos, todos en release:

| | Resultado antes |
|---|---|
| Pago con prueba de **32 ceros** | ⚠️ **Se aplica** |
| Estado del titular **mentido** (10× el saldo) | ⚠️ **Escribe la mentira** |
| Pendiente inflado (entrada 40) | ⚠️ **Rompe la conservacion** |
| **Gastar sin la clave del titular** | ⚠️ **Robo consumado** |
| **Cobrar sin la clave del receptor** | ⚠️ **Cobro ajeno consumado** |

El cuarto es el que decide la gravedad: Mallory, sin conocer `SK_ALICE`,
debito 250.000 de su cuenta y creo un pendiente dirigido a **su propia
identidad**, con una prueba de 32 ceros. La victima quedo en 750.000 y el
dinero, en transito hacia la atacante.

`apply_send` tenia **exactamente cuatro** condiciones de rechazo: las dos
raices vigentes, la de congelados, el limite regulatorio declarado, y que
las raices nuevas cuadraran con lo que la propia capa recomputa. Todas
sobre datos publicos que el atacante puede recomputar. **Ninguna era una
clave. Ninguna era una prueba.**

### 73.2 Lo que esto le hace a la afirmacion central del proyecto

`PAPER.md` §3 dice: *«La clave de gasto no sale de la maquina del
cliente»*. Seguia siendo cierto, y era **irrelevante**: no hacia falta que
saliera porque **no hacia falta en absoluto**.

La propiedad la demuestra el circuito y **la capa no la imponia**, porque
no lo consultaba. El test que §33 cita como demostracion lleva escrito
`// ===== 3. LA CAPA VERIFICA Y APLICA. =====` y cierra con *«lo unico que
recibe son pruebas que verifica»*. El comentario de `apply_send` decia *«el
saldo se verifica contra la hoja»*. Las tres afirmaciones eran ciertas
**solo si la prueba se verificaba**.

⚠️ Y los tres preprints publicados con DOI citan esta propiedad como el
argumento institucional central. Va con la entrada 28, que ahora tiene una
razon mayor que las tres que ya acumulaba.

### 73.3 Como se le escapo, y es el error de §32 otra vez

En el flujo honesto `send()` genera la prueba **dentro de la capa**, asi
que al llegar a `apply_send` ya era valida. La verificacion parecia
redundante.

Es contar el flujo previsto en vez de la superficie expuesta —lo mismo que
§32, donde se contaron llamadas a `transfer()` sin ver la via del cliente
que no la llamaba—.

⚠️ **Y el autor habia visto la puerta.** El comentario de `apply_send` dice
literalmente que *«quien tenga su propia prueba puede llamar directamente a
`apply_send`»*. Se documento el vector y no se cerro.

> **Documentar una puerta abierta no la cierra.** Ninguna de las
> herramientas del proyecto podia detectar esto: no es una restriccion mal
> escrita, es una llamada que no esta.

### 73.4 La correccion, y lo que NO cierra

Las tres funciones verifican ahora su prueba **antes de tocar el estado**,
con el patron que ya existia siete veces en el repositorio. Y `apply_claim`
comprueba ademas `frozen_root`, que `apply_send` si comprobaba y ella no:
sin eso valdria una raiz vieja de cuando el titular aun no estaba
congelado.

Los cinco testigos pasan de rojo a verde. **200 tests, ninguna regresion**:
la via honesta no dependia de que no se verificara.

⚠️ **Pero la entrada 40 sigue abierta, y un test en verde no debe hacer
creer lo contrario.** Se anadio a `apply_mint_to_pending` la exigencia de
que el aviso declare el mismo importe que la prueba. Eso es **mitigacion
parcial**: cierra la construccion del testigo, no la clase. La capa no
puede cerrarla del todo porque **no conoce la identidad del receptor** —esa
es la privacidad del diseño— y por tanto no puede recomputar el compromiso.

Queda abierto, **sin medir**: depositar un compromiso de un millon
declarando doscientos cincuenta mil —aviso y prueba coherentes entre si— y
cobrarlo con el aviso verdadero. `apply_claim` solo hace
`pending_amounts.remove(...)`: no contrasta el importe del aviso contra lo
que se registro al depositar.

Por eso el testigo se **renombro**: se llamaba
`..._rejects_an_inflated_pending` y mide algo mas estrecho. Un nombre
generoso en verde miente, y aqui mentiria sobre un fallo de conservacion.

### 73.5 Lo que este hallazgo dice del metodo

Salio de leer `apply_mint_to_pending` para escribir **otra cosa**. No lo
encontro ninguna herramienta, ningun barrido y ninguno de los 195 tests que
habia: la suite entera se ejecutaba por el flujo honesto, donde la prueba
siempre era valida.

Es el argumento de la **entrada 7** en su forma mas cruda. Y el proyecto
lleva dos dias diciendo que hace falta que lo mire alguien mas.

## 74. La entrada 40 es CONSERVACION ROTA, y el analisis del arreglo

⚠️ Medido el 31-07-2026, **despues** de la correccion de §73: sigue siendo
explotable de punta a punta.

### 74.1 Que se midio

| | |
|---|---|
| Suministro emitido | **250.000** |
| En cuentas al final | **1.000.000** |
| En transito | 0 |
| Resultado del deposito | **`Ok(())`** |

El camino, en dos pasos:

1. Depositar un compromiso que vale un millon **declarando doscientos
   cincuenta mil**, con el aviso coherente con la prueba —asi la mitigacion
   de §73.4 no salta—.
2. Cobrarlo con el aviso **verdadero**.

⚠️ **Y esto es lo que hace el hallazgo distinto de los anteriores.** El
primer paso paso con la prueba **ya verificada**. El segundo **no falsifica
nada**: el compromiso del millon esta de verdad en el arbol, asi que el
circuito de cobro lo reconstruye y su prueba es legitima.

> **No hay ningun paso fraudulento que un verificador pueda detectar. El
> fraude es que el circuito nunca ato las dos cosas.**

Es dinero que el tope declarado no cuenta, y el tope es la propiedad que
§66 y §67 se tomaron el trabajo de demostrar.

### 74.2 El arreglo minimo NO basta, y conviene verlo antes de cortar

Lo evidente es mover `C_PEND_IN` y `C_PEND_VAL` del carril A —cuyo digest
`C_PEND_ENTRY_A` descarta— al carril B, que es el que se inserta (§72.2).

**No basta.** `C_PEND_VAL` fija hoy `next[4..8]` —el digest interno— y
`next[8]` —el importe—, y deja libres:

- `next[0..4]`, la capacidad;
- `next[9..12]`, el relleno.

**Siete elementos.** Cambiar de carril sin cerrarlos deja un compromiso que
sigue sin estar determinado por sus entradas: distintos rellenos dan
distintas hojas para el mismo `(identidad, aleatorio, importe)`.

Seria completar el trabajo sin alcanzar el objetivo, que es exactamente la
leccion de §32 —*«un plan expresado en unidades de trabajo puede
completarse sin alcanzar su objetivo»*—.

### 74.3 Dimensiones del arreglo

| | Ahora | Tras el arreglo |
|---|---|---|
| `C_PEND_IN` | 12 ranuras, **carril A** | 12 ranuras, **carril B** |
| `C_PEND_VAL` | **5** ranuras, carril A | **12** ranuras, carril B |
| `NUM_CONSTRAINTS` en `_climb` | 89 | **96** |
| `NUM_CONSTRAINTS` en produccion | 125 | **132** |

Las siete nuevas son grado 1 con ciclo, el mismo grupo que las que ya hay:
el bucle de grados pasa de `0..17` a `0..24`. Hay que hacerlo en **los dos
circuitos**.

⚠️ **Lo que este analisis no dice**: si el carril A queda entonces sin nada
que probar en las filas 40-55. Si es asi, sobra su computo entero y habria
que decidir si se retira —lo que cambiaria el indicador de hash y las
periodicas— o se deja como esta. Sin verificar.

### 74.4 El criterio de exito

`a_pending_worth_more_than_declared_cannot_be_claimed` en verde, y con el
los dos testigos de §72 —que quedan rojos con `#[ignore]` hasta entonces—.

Que compile no es el criterio. Que los dieciseis tests de los dos circuitos
sigan pasando, tampoco por si solo: **ninguno de ellos veia esto**.

### 74.5 Por que se escribe antes de cortar

Es lo que §68 hizo con `mint_pending`, y §70.3 registro el resultado: fue
la segunda vez en el proyecto que el trabajo previo **evito** una ronda en
vez de causarla.

Y hoy se ha comprobado dos veces que lo que se pierde de una sesion no es
el codigo, es la memoria.

## 75. Entrada 40 corregida: el compromiso, en el carril que se lee

✅ **Corregida y verificada** el 31-07-2026, en los dos circuitos.

### 75.1 El arreglo

| | Antes | Ahora |
|---|---|---|
| `C_PEND_IN` | 12 ranuras, **carril A** | 12 ranuras, **carril B** |
| `C_PEND_VAL` | **5** ranuras, carril A | **12** ranuras, carril B |
| `NUM_CONSTRAINTS` en `_climb` | 89 | **96** |
| `NUM_CONSTRAINTS` en produccion | 125 | **132** |

Las doce de `C_PEND_VAL` son capacidad (4), digest interno (4), importe (1)
y **relleno (3)**.

⚠️ **El relleno era la mitad del fallo**, y por eso §74.2 se escribio antes
de cortar. Sin fijarlo, dos trazas con el mismo
`(identidad, aleatorio, importe)` producen hojas distintas: el compromiso no
queda determinado por sus entradas ni aunque se cambie de carril. Un arreglo
que solo hubiera movido las restricciones habria dejado el fallo vivo con
otra forma —el error de §32 en su version mas cara: **completar el trabajo
sin alcanzar el objetivo**—.

### 75.2 Lo que se midio

| | Antes | Ahora |
|---|---|---|
| `a_lane_b_...` (`_climb`) | 🔴 | ✅ |
| `a_lane_b_...` (produccion) | 🔴 | ✅ |
| `a_pending_worth_more_than_declared_cannot_be_claimed` | 🔴 | ✅ |
| `a_valid_mint_pending_climb_verifies` | ✅ | ✅ |

⚠️ **El positivo se corrio primero y por separado.** Tres testigos negativos
en verde no dicen nada si el circuito rechaza todo, y esta sesion ya vio esa
trampa en §66.2. Con el positivo en pie, el verde de los tres significa lo
que dice.

**272 tests en `stark-experiment` y 201 en `zk-ssl`, ninguno ignorado**, y
cero avisos con los tres ficheros recompilados a la fuerza. No queda un solo
test marcado en el proyecto.

Los tres testigos **pierden la marca y se quedan** como regresion
permanente, que es lo que §50 dejo dicho que se hace cuando el fallo que un
testigo nombra se corrige. El de la capa es hoy el unico test del proyecto
que comprueba la conservacion de punta a punta atravesando emision, deposito
y cobro.

### 75.3 `build_trace` no cambio, y eso dice algo

La traza honesta ya cumplia lo que ahora se exige: ponia los dos carriles
identicos y con capacidad y relleno a cero.

> **Las restricciones describian mal algo que la construccion ya hacia
> bien.** Por eso ningun test lo veia: todos pasaban por `build_trace`, y
> `build_trace` era honesto. Solo un testigo que **no** use la construccion
> normal puede encontrar esta clase de fallo.

Es el argumento del test discriminante (§40.4, entrada 7) con una
demostracion mas.

### 75.4 Lo que queda, declarado

El carril A sigue calculando el compromiso en las filas 40-55 y **ya no lo
lee nadie**: `C_PEND_ENTRY_A` fuerza su hoja a cero, y las restricciones se
han ido al carril B.

Es **computo muerto, no un fallo**: la fila 56 del carril A queda
determinada por `C_PEND_CAP`, `C_PEND_ENTRY_A` y `C_PEND_SIBLING` sea cual
sea lo que traiga. Retirarlo tocaria el indicador de hash y la cadena de
periodicas —entrada 39— por una ganancia de cero filas. **Se deja y se
declara**, en vez de arreglarlo de paso en un commit que corrige otra cosa.

### 75.5 Lo que este arreglo NO cierra

Que `circuit_mint_pending` sea correcto. Cierra **el fallo que se midio**.

La clase a la que pertenecia —restriccion ausente, o escrita sobre el sitio
equivocado— no la detecta ninguna herramienta del proyecto por
construccion: `check_constraint_layout.py` no vio colision, desborde ni
ranura muerta, y `buscar_vacias` daba las 89 disparando. **Las dos tenian
razon**: el reparto estaba bien y ninguna restriccion era vacua. El problema
era donde miraba una de ellas.

Y este fallo salio de leer el circuito para escribir otra cosa. Es la
tercera vez hoy.

## 76. El README publicaba una afirmacion falsa, y no desde hoy

La entrada 2 se abrio por cifras rancias y se cerro midiendolas. Al volver
a medirlas el 31-07-2026 aparecio algo peor que una cifra.

### 76.1 Lo que decia y lo que hay

> *«El crate de circuitos **si pasa** en los dos modos.»*

~~**No pasa.**~~ ✅ **Pasa desde el mismo dia, una hora despues**: los tres
se diagnosticaron y corrigieron en §77, y el README se puso al dia en el
mismo commit. Lo que sigue siendo cierto es que **cuando se escribio esa
frase era falsa**, y lo era desde antes.

⚠️ **Y el rodeo dice algo que conviene no perder.** La correccion del README
envejecio en una hora **porque el propio texto llevo a arreglar lo que
denunciaba**. Escribir «no pasa, y estos son los tres» convirtio una
afirmacion comoda en una tarea con nombre. Es el mejor argumento que va a
tener este proyecto para medir las cifras en vez de recordarlas.

En depuracion fallaban tres tests de solidez de custodios:

- `circuit_threshold_single::the_index_test_rejects_for_the_right_reason`
- `circuit_threshold_single_nullifier::an_attacker_cannot_bring_their_own_custodian_set`
- `circuit_threshold_single_nullifier::one_real_and_one_forged_custodian_do_not_meet_the_threshold`

Y **ya fallaban antes de esta sesion**: volviendo al commit `44c5b8c` fallan
exactamente los mismos tres.

> **No es una cifra rancia: es una afirmacion de estado contradicha por el
> propio comando que el README invita a ejecutar.** Una cifra desfasada
> envejece sola; esta habia que escribirla.

### 76.2 Lo medido, contra lo publicado

| | Publicaba | Medido 31-07 | Antes de la sesion |
|---|---|---|---|
| `stark-experiment` release | 201 | **272** | 252 + 3 fallos en debug |
| `zk-ssl` release | 174 | **201** | — |
| Fallos de `zk-ssl` en debug | 65 | **80** | **77** |
| Circuitos | «doce» | **27** | — |
| Con prueba de vacuidad | 11 de 12 | **26 de 27** | — |

⚠️ **Los 65 tampoco eran de hoy.** Eran 77 antes de la sesion. Sustituirlos
por 80 sin decirlo habria ocultado que el desfase venia de antes, que es
justo lo que la entrada 2 castiga.

### 76.3 Lo que NO se toco, y por que

La tabla «Que corrige la tercera revision» dice que corrigio **esa**
revision respecto de la anterior. Sus cifras son las de entonces y
**siguen siendo correctas para lo que afirman**.

Actualizarlas habria reescrito lo que se publico. Se le añade una nota que
declara que es historica, y se deja.

> **Una cifra vieja en un contexto historico no esta rancia: esta fechada.**
> Distinguir las dos cosas es la diferencia entre corregir un documento y
> falsificarlo.

### 76.4 Los tres tests quedan SIN diagnosticar

Sus nombres son de solidez, no de rendimiento. Podrian ser la clase de la
entrada 6 —grado dependiente del testigo— o podrian no serlo.

**No se atribuyen**, porque hoy ha pasado dos veces que una atribucion sin
comprobar era falsa. Van como **entrada 44**, con el metodo que ha
funcionado: diffear el vector de grados, aislar la variable, test
discriminante.

⚠️ Y hay algo que incomoda en ellos y conviene decir: son los tres tests
que comprueban que **un custodio falso no alcanza el umbral** y que **el
indice no se puede mentir**. Que lleven fallando en depuracion sin que
conste en ninguna parte es exactamente lo que la entrada 7 dice que las
herramientas del proyecto no cubren.

## 77. Entrada 44: un sintoma, dos causas, arreglos opuestos

Los tres tests de solidez de custodios que fallaban en depuracion desde
antes de la sesion (§76.1). Medidos el 31-07-2026.

### 77.1 El primero no era un fallo

`the_index_test_rejects_for_the_right_reason` construye a proposito una
traza que viola `C_ACC_FINAL` —prueba el camino del custodio 2 declarando
el indice 3— y afirmaba que la rechaza el **verificador**.

En depuracion la rechaza el **probador**, y con mas detalle: *«main
transition constraint 23 did not evaluate to ZERO at step 39»*. El test
recibia `FallaAlProbar` donde esperaba `FallaAlVerificar`.

> **Los dos comportamientos son correctos.** Lo que estaba mal era una
> afirmacion escrita para un modo y comprobada en los dos.

Arreglo: esperar lo que corresponde a cada modo. **Se gana cobertura**: el
test vuelve a correr en depuracion, donde ademas nombra la restriccion y el
paso.

### 77.2 Los otros dos: el indice 0, otra vez

`an_attacker_cannot_bring_their_own_custodian_set` y
`one_real_and_one_forged_custodian_do_not_meet_the_threshold` usaban
`paths_mias[0]`. El vector de grados lo dice sin ambiguedad:

| indices | que son | declarado | indice 0 |
|---|---|---|---|
| 16-19 | `C_PLACE` | 126 | **63** |
| 20 | `C_BIT_BOOL` | 63 | **0** |

Es la entrada 6: el camino del indice 0 tiene **todos los bits de direccion
a cero**.

### 77.3 ⚠️ El arreglo es el OPUESTO al de §71.3, y el criterio importa

En §71.3 se decidio **no** cambiar la posicion para que el test pasara,
porque la posicion 0 **es la que produccion usa** —`allocate_pending`
reutiliza huecos y un ledger recae en ella (§46.1)—. Un test en otra
posicion habria pasado sin ejercitar el caso comun.

Aqui el indice del atacante es **incidental**: lo que se comprueba es que
trae otro conjunto, no que use el hueco 0. Cambiarlo no debilita nada y
devuelve los dos tests a depuracion.

> **El criterio no es «evitar el caso degenerado» ni «declararlo»: es si el
> caso degenerado es el que produccion ejecuta.** Mismo sintoma, misma
> causa, decisiones contrarias.

### 77.4 Y el proyecto ya lo sabia

`tests_support.rs` lleva escrito desde antes: *«Indices 1 y 3: el 0 tiene
todos los bits de camino a cero y degeneraria la traza»*. Y en el propio
`circuit_threshold_single_nullifier`, **todos los demas tests usan los
indices 1 y 2**. Solo estos dos usaban el 0.

Es el patron de §59.2, §62.2 y la entrada 39 por cuarta vez:
**conocimiento que existe y no se aplica a todo el codigo**. Aqui ni
siquiera era una herramienta con cobertura parcial: era un comentario en
otro fichero.

### 77.5 Lo que esto NO era

Un fallo de solidez. Los tres tests pasaban en release y lo que comprueban
—que un conjunto de custodios ajeno no cuela y que el indice no se puede
mentir— **siempre se cumplio**.

Lo que fallaba era la instrumentacion, y llevaba fallando lo bastante como
para que el README publicara que el crate pasaba en los dos modos. Un test
rojo permanente que nadie mira es un test que no existe, y ademas **tapa a
los que aparezcan al lado**.

## 78. Entrada 41: los ochenta, clasificados

No se diagnostican contando: se clasifican por **clase de panico**. Medido
el 31-07-2026 sobre la salida completa.

### 78.1 Dos clases, y una no era la conocida

| | ruta en winterfell | mensaje | que es |
|---|---|---|---|
| **78** | `evaluation_table.rs` | `degrees didn't match` | Grado declarado que no se realiza |
| **2** | `trace/mod.rs` | `trace does not satisfy assertion` | ⚠️ **Una asercion que la traza no cumple** |

**No son lo mismo.** El primero es un limite de la herramienta —la
restriccion impone lo que debe, y release genera y verifica bien—. El
segundo es una traza que no cumple lo que se le exige, y eso **en un camino
legitimo seria un fallo**.

Contarlos como «ochenta del limite de grados» habria enterrado los dos.

### 78.2 Los dos: escenarios de rechazo

`main_trace(16, 39)`: la columna 16 es `LANE_B + 4` y la fila 39 es
`ROW_ROOT` —la asercion de que **el segundo carril llega a la raiz del
conjunto**, o sea, que el segundo firmante pertenece a el—.

- `a_custodian_cannot_change_the_custodian_set`
- `the_governance_set_survives_restart`

Los dos construyen un **impostor**: claves de custodio recorriendo caminos
de gobernanza. Su carril no llega a esa raiz, y por eso se rechazan. **Es
la propiedad que prueban.**

⚠️ **El segundo estaba escondido detras de su nombre.** «Sobrevive al
reinicio» suena a camino legitimo; la propiedad esta en el negativo que
lleva dentro, doce lineas mas abajo. Solo aparecio al clasificar por clase
de panico.

> **Un test cuyo nombre describe la mitad positiva esconde la negativa en
> los informes de fallo.**

### 78.3 Por que no se arreglan como §77.1

Alli el test comparaba `Rechazo::FallaAlProbar` contra
`Rechazo::FallaAlVerificar` y bastaba con esperar el que corresponde a cada
modo.

Aqui el panico ocurre **dentro de la capa**, en `update_custodians`, que
llama a `prove`. **Una capa no puede capturar un panico para devolver un
`Err`**, y envolverlo en `catch_unwind` cambiaria lo que el test comprueba
—§45 retiro treinta y dos de esos bloques por meter una carrera—.

Se marcan con `cfg_attr(debug_assertions, ignore)` y el motivo medido.
Release los ejecuta.

### 78.4 Los 78 NO se marcan, y es una decision

Podrian marcarse en bloque y dejar la depuracion en verde. **No se hace**,
por dos razones:

1. El motivo real **varia**: indice de cuenta 0 y 1, arbol de congelados
   vacio, margen del tope a cero (entradas 6, 24, 25). Ponerles a los 78 el
   mismo texto seria atribuir en bloque, que es exactamente el error que
   §77 acaba de desmentir —un sintoma, dos causas—.
2. La entrada 44 enseño que **cambiar el testigo puede ser mejor que
   marcarlo**, y el criterio es si el caso degenerado es el que produccion
   ejecuta (§77.3). Eso hay que decidirlo test a test, no de golpe.

⚠️ **Y el precio de no marcarlos queda dicho**: mientras haya 78 rojos
permanentes, **un fallo nuevo en depuracion se esconde entre ellos**. Es lo
que le paso a la entrada 44 durante quien sabe cuanto tiempo.

### 78.5 Lo que esta entrada gana hoy

Pasa de «80 fallos sin diagnosticar» a «78 de clase conocida y decidida, y
2 de diseño de test, corregidos». Lo que queda es trabajo dimensionado, no
una incognita.

## 79. Un fallo de 1 de 12, y como aparecio

Release estaba en verde y llevaba todo el dia en verde. Una pasada suelta,
corrida por otro motivo, dio **200 y 1 fallo**. Las tres siguientes, 201.

### 79.1 Un error mio antes del hallazgo

Al pedir el nombre del test lo puse en una pasada distinta de la que
fallaba, y esa paso. **Se perdio el nombre por ordenar mal los comandos.**

Queda anotado porque el reflejo natural entonces es «habra sido cosa de una
vez». Un fallo unico sin explicar es exactamente como empezo §29, y alli la
primera hipotesis se descarto **mal** con `--test-threads=1` —un hilo
elimina la carrera que se quiere probar—.

### 79.2 Reproducido subiendo la contencion

Doce pasadas a 16 hilos, guardando cada salida: **1 de 12**.

```
tests::an_encrypted_ledger_needs_the_right_passphrase
panicked at tests.rs:444
  Store(Io("could not acquire lock on \"/tmp/zkssl_encrypted_43267/db\":
  WouldBlock"))
```

Es el **bloqueo de directorio de `sled`** —entrada 18, §16.6— con
manifestacion medida por primera vez. Y el numero de linea importa: **444
es la reapertura con la contraseña CORRECTA**, no la comprobacion de la
incorrecta.

> ⚠️ **No era un fallo de seguridad, y habia que comprobarlo.** Ese test
> tiene dos formas de fallar: una es el bloqueo, y la otra seria que **una
> contraseña equivocada abriera el ledger cifrado**. La segunda habria sido
> el quinto hallazgo del dia. Leer el mensaje, y no el nombre del test, es
> lo que las distingue.

### 79.3 La proteccion existia, en 39 sitios de 48

`open_retry` se escribio precisamente para esto y lo usaban **39**
llamadas. Las **9** que abren el ledger cifrado iban directas a
`SovereignLayer::open_encrypted`, sin red.

Es el patron de §59.2, §62.2, §66.2 y §77.4 por **quinta vez en la
sesion**: algo util aplicado a parte del codigo sin que conste a que parte.
Y aqui ni siquiera era una herramienta: era una funcion en el mismo crate.

Correccion: `open_encrypted_retry`, hermano exacto, en las **nueve**. Solo
absorbe errores de E/S; cualquier otro —incluida la contraseña equivocada—
se devuelve de inmediato y no puede quedar enmascarado.

### 79.4 ⚠️ Doce en verde NO demuestran que este arreglado

Demuestran que **no aparecio en doce**, en las mismas condiciones en que
antes aparecia una de cada doce. Es evidencia, no prueba.

Lo que si sostiene la correccion es que el remedio es el que `open_retry`
lleva usando en 39 llamadas sin incidencia registrada.

⚠️ **Y el fondo sigue abierto**: la entrada 18 no se cierra. Esto arregla
los **tests**; un nodo real que se reinicie inmediatamente tras cerrarse
puede sufrir lo mismo, y ahi no hay `open_retry` que valga.

### 79.5 Lo que ensena sobre el verde

Los 78 rojos permanentes de depuracion esconden fallos nuevos **por
ruido** —le paso a la entrada 44—. Este ha ensenado que el verde de release
tambien esconde, **por escasez**: un fallo de 1 de 12 no aparece en la
pasada de antes de commitear.

> **Una suite en verde no dice que no haya fallos: dice que no salieron
> esta vez.** La diferencia solo se ve corriendola muchas veces, y eso no
> se hace nunca porque en verde no parece hacer falta.

## 80. Retirar las vias antiguas: el inventario antes de cortar

§65.4 fija que se retiran «cuando las cinco operaciones tengan via
delegada». Eso se cumplio el 31-07-2026 (§71). Esto mide **que costaria**,
antes de tocar nada.

### 80.1 Lo que ya se ha hecho, y era la mitad que faltaba

§65 marco **tres de cinco**. `mint` y `mint_to_pending` quedaron sin marca
**precisamente porque les faltaba la via delegada**.

Y las tres notas existentes decian: *«Se conserva mientras `mint` y
`mint_pending` no tengan equivalente delegado»*. **Esa condicion se cumplio
esta mañana**, asi que las tres estaban rancias desde entonces.

Corregido: las **cinco** marcadas, con la nota al dia. Cero avisos, porque
§65.3 se respeta —el permiso va en los tests— y `metrics.rs` y
`tests_support.rs`, que no lo tenian, lo llevan ahora con el motivo escrito.

### 80.2 El inventario

| via antigua | llamadas | donde |
|---|---|---|
| `mint` / `apply_mint` | **54** | tests, metrics, tests_support |
| `set_frozen` / `apply_freeze` | 38 | tests, iso, snapshot |
| `update_custodians` / `apply_governance` | 19 | tests |
| `recover` / `apply_recovery` | 18 | tests |
| `mint_to_pending` / `apply_mint_to_pending` | 9 | tests |
| | **138** | |

⚠️ **Y el numero que de verdad manda no esta en esa tabla.**
`open_and_fund` —el unico camino legitimo para que una cuenta tenga saldo—
llama a `.mint()` por dentro, y **se usa 145 veces**.

> **La mitad de la suite depende de la via antigua sin nombrarla.** Un
> barrido por nombre de funcion habria contado 54 y se habria dejado 145.
> Es literalmente el error de §32: contar llamadas a una funcion no mide
> cuanta superficie depende de lo que esa funcion usa.

### 80.3 El precio, y por que no esta medido

La via antigua genera **una** prueba —`circuit_mint`, con los custodios
dentro—. La delegada genera **tres**: la subida mas dos de umbral.

Migrar `open_and_fund` multiplica por tres el coste de fondear una cuenta,
145 veces. La suite esta hoy en **31 s** en release y **250 s** en
depuracion.

⚠️ **Cuanto sube exactamente no se sabe.** Puede ser el doble o puede ser
diez veces, y de eso depende que la retirada sea viable o que el precio
razonable sea dejar las cinco marcadas sin retirarlas.

### 80.4 Y hay un efecto que no es de coste

`metrics.rs` mide **la via que se ejecuta**. Si `open_and_fund` cambia, las
cifras de rendimiento publicadas cambian con ella —y no porque el sistema
sea mas lento, sino porque se estaria midiendo otra cosa—.

Eso hay que **declararlo**, no absorberlo. La marca `allow(deprecated)` que
lleva ahora ese modulo lo dice en el codigo.

### 80.5 El experimento que decide

Migrar **solo** `open_and_fund` a la via delegada, en una rama, y
cronometrar la suite en los dos modos. Un cambio, un numero.

Con ese numero se decide entre:

- **retirar**, si el coste es asumible;
- **no retirar y declararlo**, dejando las cinco marcadas y escribiendo por
  que —que es lo que §46 hizo con la entrada 6—.

⚠️ **Lo que no se puede hacer es empezar a borrar y ver que pasa.** §32
—la entrada que da nombre a esto— es el registro de un plan que se
completo en unidades de trabajo sin alcanzar su objetivo. Aqui el objetivo
no es borrar cinco funciones: es que nadie pueda pedir las claves.

### 80.6 Lo que la marca NO da

§65.5 ya lo decia y sigue valiendo: `#[deprecated]` es un aviso. Nada
impide llamarlas con un `#[allow]`, y este mismo commit añade dos.

**La garantia la da usar la via delegada.** Mientras las antiguas existan,
el fallo de la entrada 32 esta *evitable*, no *cerrado*.

## 81. Entrada 39 cerrada, y los tres errores que costo cerrarla

`check_constraint_layout.py` cruzaba los indices de `result[...]` y **no los
de `periodic[...]`**. Son dos arrays y miraba uno.

### 81.1 Por que importaba

En `mint_climb` quedaron tres constantes `P_*` muertas, `P_SEG_LINK` se
desplazo y el indice se salio del array (§66.2). **Se noto porque
desbordo.**

Si el desplazamiento fuera hacia abajo, la restriccion leeria la columna
periodica equivocada **en silencio** —activandose en las filas que no son—.
Ni las colisiones, ni las ranuras muertas, ni `buscar_vacias` lo verian: es
la clase de §39 y §72.

Ahora comprueba `DESBORDE PERIODICA` —se lee por encima de lo construido— y
`MUERTA PERIODICA` —se construye y nadie lee—, con autotest propio.

### 81.2 Reproduce sola una comprobacion que se hizo a mano

Da **32** en `circuit_mint_pending_climb` y **41** en
`circuit_mint_pending`: exactamente lo que hubo que contar a mano esta
mañana al amputar, y donde §68 se equivoco contando ocho periodicas cuando
eran nueve (§70.1).

> Automatizar lo que ya se hace a mano es la conversion mas barata que hay,
> y la unica que no se olvida de hacerse.

### 81.3 ⚠️ Los TRES errores que costo, porque valen mas que la herramienta

**Uno.** La primera regex casaba `periodic[` y no `periodic_values[`. Seis
circuitos usan el segundo nombre y el barrido reporto **159 columnas muertas
inexistentes**.

> Es **literalmente el mismo agujero que `RE_WRITE` tenia antes de §59.2**,
> cometido al cerrarlo. El patron no es que se olvide: es que se repite en
> quien lo esta corrigiendo.

**Dos.** El contador devolvia `0` cuando no entendia la construccion.
`solvency` devuelve `vec![a, b, c]` en vez de usar `push`, y aparecio con
seis desbordes falsos.

La regla de §42.5 decia que un barrido no debe **aprobar** lo que no
entiende. No decia lo otro, y hacia falta:

> **Un barrido que CONDENA lo que no entiende es tan malo como uno que lo
> aprueba.** Los dos mienten; el segundo ademas entrena a ignorarlo.

**Tres.** Un `\b` de expresion regular atraveso dos capas de comillas y se
colapso en un caracter de **retroceso** (0x08). El patron dejo de casar, el
contador dio cero en los 27 circuitos y el barrido reporto **823 desbordes**.
Sustituido por busqueda de texto plano: sin escapes no hay nada que
colapsar.

### 81.4 Lo que los tres tienen en comun

Los tres **compilaban**, ninguno lanzaba excepcion, y los tres producian un
informe con aspecto de informe. El primero y el tercero habrian pasado por
buenos si el resumen se hubiera leido por encima; el segundo habria hecho
que alguien fuera a «arreglar» `solvency`, que no tenia nada roto.

> **Una herramienta de auditoria que se equivoca no falla: miente con
> autoridad.** Por eso lleva autotest, y por eso el autotest se escribio con
> un caso que ya habia fallado de verdad.

Los tres los cazo ejecutarla contra los 27 circuitos antes de darla por
buena. Ninguno lo habria cazado leerla.

## 82. El espacio de claves: 64 bits, medido

La entrada 15 decia, en dos lineas y sin referencia a ninguna seccion:
*«Goldilocks es estrecho para identidades: 64 bits son colision en 2^32»*.

### 82.1 Lo que decia ya estaba corregido

La cabecera de `circuit_settlement.rs` documenta ese hallazgo **y su
arreglo**: si la identidad fuera un solo elemento, encontrar `sk'` con la
misma `pk` costaria ~2^32 por la paradoja del cumpleaños. Por eso la
identidad paso a ser el **digest completo de 4 elementos, 256 bits**.

La entrada describia el problema *antes* del arreglo, y llevaba asi lo
bastante como para que nadie la releyera.

### 82.2 Lo que la entrada NO decia, y sigue vivo

**El SECRETO sigue siendo un solo elemento.**

```rust
pub fn open_account(&mut self, spend_key: BaseElement) -> AccountIndex
pub fn derive_public_id(spend_key: BaseElement) -> Digest
pub fn derive_governor_id(key: BaseElement) -> Digest
```

`pk = Rescue(DOMAIN, sk)` con `sk` en Goldilocks: **2^64**. Y `pk` es
**publica** —`public_id_of` es `pub`, y el puente ISO la lleva en los
mensajes porque el pagador la necesita para direccionar—.

El ataque no es una colision: es **busqueda exhaustiva fuera de linea**.
Dado `pk`, enumerar `sk` y comparar. Coste esperado **2^63**, sin limite de
intentos porque no toca el sistema.

> ⚠️ **Ensanchar la identidad a 256 bits no ayuda contra esto.** Impide
> encontrar OTRA clave con la misma identidad; no impide encontrar LA clave.
> Son dos problemas distintos y solo se arreglo uno.

### 82.3 La medida

`el_coste_de_agotar_el_espacio_de_claves`, en release:

| | |
|---|---|
| `derive_public_id` por segundo y nucleo | **122.850** |
| 2^63 evaluaciones | 9,223 × 10^18 |
| **Años-nucleo** | **2.379.098** |
| Con 1.000 nucleos | 2.379 años |
| Con 100.000 nucleos | **23,8 años** |
| Con 10.000.000 nucleos | **87 dias** |

⚠️ **Es una cota superior floja del coste real.** Ese numero sale de CPU sin
optimizar el ataque: con asignacion de memoria por llamada, rehaciendo la
mitad constante del estado en cada intento, y comparando el digest entero
antes de descartar. Un atacante usaria GPU o ASIC y ninguna de las tres.
**El margen real es menor que el medido, no mayor.**

### 82.4 ⚠️ El paper ya tiene el argumento, y no lo aplica aqui

`PAPER.md` §8.3, sobre el techo de solidez del STARK:

> *«63 bits de solidez, **insuficiente y no comparable con los ~128 bits**
> de los otros paradigmas.»*

**Ese mismo criterio se aplica al espacio de claves, y el paper no lo
hace.** Son dos consecuencias independientes de la misma estrechez de
Goldilocks:

| | consecuencia | estado |
|---|---|---|
| Solidez del STARK | techo de 63 bits | Documentado §16.7, **publicado** |
| Espacio de claves | 64 bits | ⚠️ **En ninguna parte** |

Arreglar una no arregla la otra. Y quien lea el paper concluira que la
estrechez de Goldilocks tiene **una** consecuencia, porque es la unica que
se nombra.

### 82.5 Que haria falta, y su tamaño

`sk` de **cuatro elementos** (256 bits). Toca `derive_public_id`,
`native_nullifier`, `derive_governor_id`, `derive_custodian_id` y todos los
circuitos que absorben la clave —`send`, `claim`, `burn`, `audit`,
`threshold`—.

**No se hace hoy** y no se propone a la ligera: es un cambio de formato de
~~identidad que invalida cualquier cuenta existente.~~ **⚠️ Rectificado en
§90 (31-07-2026): NO invalida ninguna cuenta.** Rellenar una clave estrecha
con ceros da **la misma identidad** —hay test—, asi que la version ancha es
una generalizacion, no un reemplazo. Lo que si se hace es **dejar de
llamarlo «colision en 2^32»** y dejarlo medido.

### 82.6 Lo que este caso enseña del backlog

La entrada 15 tenia dos lineas y **era la unica sin referencia a una
seccion**. Eso era el sintoma: nada la respaldaba porque nunca se analizo.

> **Una entrada de backlog sin analisis detras no es una tarea pendiente:
> es una intuicion vieja disfrazada de tarea.** Y disfrazada de tarea, se
> lee por encima y se pospone, que es lo que le paso durante meses.

Y lo que decia era ademas **falso a estas alturas** —describia algo
corregido—, asi que releerla confirmaba que estaba «bajo control».

## 83. El backlog auditado a si mismo, y la hipotesis que fallo

La entrada 15 resulto describir un problema **ya corregido** (§82), y era la
unica sin referencia a ninguna seccion. La hipotesis era obvia: **buscar las
demas sin respaldo, porque tendran el mismo problema.**

### 83.1 Nueve, no una

| entradas abiertas sin ninguna referencia | 9 de 18 |
|---|---|

`11, 12, 16, 17, 19, 20, 22, 28, 42`.

### 83.2 ⚠️ Y revisarlas una a una absolvio a SIETE

- **22** —la que mas riesgo parecia, por llevar una cifra publicada— resulto
  estar **guardada por un test** (`cost_per_transfer_stays_stable`) con
  margenes anchos a proposito, que **ya salto una vez** en la migracion a la
  via en dos fases y tenia razon (§31). Medido hoy: 123,0 por mil, dentro de
  la guarda.
- **12** tiene su respaldo en §30 y en un test que lo ejercita.
- **11, 17, 19** son constataciones de ausencia: no hay seccion porque no
  hay nada que analizar.
- **16, 28** hablan de documentos externos, verificables contra ellos.
- **42** se escribio hoy y solo le faltaba citar §73.4.

> **La hipotesis era falsa.** «Sin seccion detras» no predice que la entrada
> sea falsa: predice que **nadie la ha releido**, que es condicion necesaria
> para envejecer mal, no suficiente. La 15 lo era por otra razon —describia
> un arreglo ya hecho—.

Se registra porque un barrido que confirma lo que se esperaba enseña menos
que uno que lo desmiente, y porque la tentacion al ver «9 de 18 sin
respaldo» era anunciar nueve problemas.

### 83.3 Lo unico accionable que aparecio: una unidad

El arnes de metricas calcula la acumulacion asi:

```rust
"...{:.1} MB acumulados", (tx_bytes * 1000) as f64 / 1_048_576.0
```

Divide entre 2^20 —**MiB**— y lo etiqueta **«MB»**. Lo mismo con «~126 KB
por pago»: medido son 129.014 B = **126,0 KiB** exactos.

La convencion es coherente en todo el proyecto **y llega a los tres
preprints**:

| lectura | valor |
|---|---|
| Medido el 31-07-2026, en MiB | **123,0** |
| Publicado (MiB implicito) | 120,4 → deriva 2,2 %, dentro de la guarda |
| **Si el lector toma MB = 10^6** | **129,0** → lo publicado dice **7,2 % menos** |

No es un error de medicion: es una etiqueta. Pero esta en material con DOI,
y va con las entradas 16 y 28.

### 83.4 La 20 era la unica irrefutable

Decia *«implementada solo en parte»*. Sin decir que parte, no se puede
confirmar ni refutar.

> **Una entrada que no se puede refutar no envejece: se queda para siempre
> pareciendo pendiente.** Es peor que una equivocada, porque la equivocada
> al menos puede corregirse cuando alguien la mire.

Descompuesta, son cuatro casos y **dos estan hechos**: rotacion de custodios
por uso, y `recover` para la clave de cuenta. Falta el recifrado del ledger.
Y la gobernanza **es inmutable por diseño**, no un hueco —aunque su coste,
que una clave de gobernanza comprometida lo esta para siempre, si merece
decision propia—.

## 84. La cuarta revision: inventario antes de tocar

La entrada 28 se abrio por una sola cosa —§27, el cobro—. El 31-07-2026 son
**cuatro**, y tres se midieron ese dia. Esto las inventaria **sin tocar los
preprints todavia**.

### 84.1 Los cuatro frentes, y no son iguales

| | frente | donde | naturaleza |
|---|---|---|---|
| **A** | La clave de gasto (§73.2) | `PAPER.md:194`, `PAPER_EN.md:186` | ⚠️ propiedad **no impuesta** hasta el 31-07 |
| **B** | El cobro (§27, §39.1) | `PAPER.md:613`, `PAPER_EN.md:578` | ⚠️ propiedad **no impuesta** hasta el 30-07 |
| **C** | 128 bits y espacio de claves (§82.4) | `PAPER.md:533`, `PAPER_EN.md:500` | analisis **que falta** |
| **D** | Unidad MiB/MB (§83.3) | 6 sitios, los tres documentos | etiqueta **equivocada** |

Mas las referencias cruzadas (entrada 16). **Once pasajes como minimo.**

### 84.2 A y B no son «el paper dice algo falso»

Las dos frases son **ciertas hoy**:

> *«La clave de gasto no sale de la maquina del titular»* — cierta desde
> §73, el 31-07.
>
> `| claim | El receptor lo hace suyo | Solo el del receptor |` — cierta
> desde §39.1, el 30-07.

Y el **diseño siempre fue correcto**. Lo que fallaba era que la
implementacion no lo imponia: en A la capa no verificaba la prueba, asi que
la clave no hacia falta para nada; en B el circuito no ataba el compromiso a
la identidad del cobrador.

> **Los preprints presentaban como propiedad DEL SISTEMA algo que solo era
> propiedad DEL DISEÑO.** Quien leyo v1, v2 o v3 —artefactos con DOI— tenia
> dos garantias que el codigo no daba.

### 84.3 La decision: se anota

⚠️ **La cuarta revision dira que esas dos propiedades no estaban impuestas
en las tres anteriores, y desde cuando lo estan.**

No es la opcion comoda. Sin anotarlo el paper queda correcto y nadie miente;
quien leyo v3 y confio en esas dos frases no se entera nunca. Anotarlo obliga
a escribir en un artefacto publico que durante tres revisiones se afirmo algo
que el codigo no garantizaba.

**Y hay precedente en el propio proyecto.** `PRINCIPIOS.md` §8 corrigio la
hoja de ruta original —testnet 2026-2027, mainnet 2027-2028— con este
motivo:

> *«Eso no es alcanzable con los recursos actuales y decirlo seria faltar al
> principio de transparencia.»*

Se hizo una vez con una prevision optimista. Lo de ahora es del mismo tipo y
mas serio, porque no es una expectativa sino una garantia de seguridad.

`PRINCIPIOS.md` cierra: *«Si el criterio es transparencia, coherencia e
imagen fiel de la realidad, esta es la descripcion exacta de lo que hay.»*
Callar A y B seria dejar de cumplir esa frase el mismo dia que se mide.

### 84.4 C es lo unico que sigue abierto

A, B y D son cosas hechas que hay que **contar**. C es un analisis **que
falta**: el paper llama al techo de 63 bits de solidez *«insuficiente y no
comparable con los ~128 bits de los otros paradigmas»* y **no aplica ese
criterio al espacio de claves**, que es de 64 bits (§82, entrada 15).

⚠️ **La cuarta revision no puede limitarse a añadirlo como nota**: si el
criterio de los 128 bits vale para la solidez, vale para las claves, y
entonces el paper tiene que decir que **el sistema no alcanza ese listón**
mientras `sk` sea un elemento. Eso no es una errata: es una conclusion.

### 84.5 D: seis sitios y una division

```rust
"...{:.1} MB acumulados", (tx_bytes * 1000) as f64 / 1_048_576.0
```

MiB etiquetado «MB», y lo mismo con «~126 KB por pago» —medido, 126,0 KiB
exactos—. En SI la cifra es **129,0 MB**, un 7,2 % mas.

`PAPER.md:487`, `PAPER.md:899`, `PAPER_EN.md:456`, `PAPER_EN.md:857`,
`QUESTIONS.md:235`, `QUESTIONS.md:251`. Y `QUESTIONS.md:72` habla de
«62 KB proofs», que es **por prueba**, no por transferencia: un pago son dos.

### 84.6 Por que el inventario va antes

Un documento publicado **no se rectifica con `git revert`**. Una vez subida
la cuarta revision a Zenodo, corregir un olvido exige una quinta.

Y hoy ha pasado tres veces que el analisis previo evito una ronda —§70.3— y
una que **lo escrito llevo a arreglar lo que denunciaba** (§76.1): el README
declaro tres tests rojos y esa misma tarde dejaron de estarlo.

## 85. Ensanchar la clave: inventario y plan, antes de escribir nada

La entrada 15 quedo medida en §82: el espacio de secretos es 2^64 porque
`sk` es **un solo elemento**. Esto inventaria que costaria pasarlo a cuatro
—256 bits— y deja el plan decidido.

### 85.1 Lo que NO hay que cambiar

Las ocho derivaciones son **una sustitucion cada una**:

```rust
merge(as_digest(DOMAIN), as_digest(sk))   →   merge(as_digest(DOMAIN), sk)
```

`derive_public_id`, `native_nullifier` ×4, `derive_governor_id`,
`derive_custodian_id` ×2. **La estructura del hash no cambia**, asi que el
nulificador tampoco necesita otra forma de absorcion. Era una de las tres
incognitas y queda descartada.

Y en el estado de Rescue **hay sitio**: la clave ocupa la ranura 8 de 12, y
9-11 estan libres.

```rust
state_a[4] = SPEND_KEY_DOMAIN;
state_a[8] = spend_key;          // → state_a[8..12].copy_from_slice(&sk)
```

⚠️ Esas ranuras libres **no son un agujero**: `C_PK_CHECK` clava la salida a
los 256 bits de `pk`, y encontrar una preimagen de eso son 2^256. Se
comprobo antes de dar el dato.

### 85.2 Lo que si cuesta, contado sobre `circuit_settlement`

| | ahora | tras ensanchar |
|---|---|---|
| `COL_S_KEY` | 1 columna | **4** |
| `TRACE_WIDTH` | 49 | **52** (+3) |
| `C_NULL_KEY` | 2 | **8** |
| `C_PK_INPUT` | 2 | **8** |
| `C_TRANSPORT` | 9 | **12** |
| `NUM_CONSTRAINTS` | **155** | **170** (+15) |

⚠️ **Son +15, no +6.** La primera estimacion conto la clave una vez y entra
**dos**: para derivar `pk` y para el nulificador. Es la tercera vez en la
sesion que una estimacion sin contar sale corta.

⚠️ **Y el tamaño de prueba NO queda igual**, como se dijo antes de mirar:
`COL_S_KEY` es una **columna de traza**, no solo una ranura del estado.
Ensanchar la clave ensancha la traza.

### 85.3 El churn, medido

| | |
|---|---|
| `derive_public_id` | **88 usos en 22 ficheros** |
| `SK_ALICE` / `SK_BOB` / `SK_MALLORY` | **329 menciones**, de las cuales **122** son `BaseElement::new(SK_*)` |
| Conjuntos de custodios y gobernanza | 9 literales en 2 funciones |
| Circuitos que absorben la clave | 5 de gasto + umbral y gobernanza |

El churn de tests es **mecanico**: se cambia el ayudante y se sustituye un
patron. No son 329 ediciones a mano.

### 85.4 ⚠️ «Empezar por un circuito» NO funciona, y conviene saber por que

El guion de las cinco amputaciones —hacer uno entero, medir, seguir— **no se
aplica aqui**. Alli cada circuito extraido era **nuevo y aislado**. Esto es
un **cambio de formato de identidad**, y `derive_public_id` la usan 22
ficheros.

En cuanto un circuito espere una `pk` derivada de cuatro elementos y la capa
la derive de uno, **dejan de coincidir** y se rompe todo lo que toque
cuentas.

> **Un formato no se migra por partes.** El error habria sido empezar por
> `circuit_burn` «porque es el mas estrecho» y descubrirlo a mitad.

### 85.5 El plan: C, luego B; A solo si B no cabe

**C — medir el coste, sin tocar la identidad.** La pregunta no es como se
ensancha una clave: es **cuanto cuestan +3 columnas y +15 restricciones en
tamaño de prueba y tiempo**. Eso se mide con columnas de relleno y
restricciones triviales sobre `circuit_settlement` —cuyo `SettlementAir`
**la capa no ejecuta**: solo lo referencia `transfer.rs`, que no esta
declarado en `lib.rs`—. Mismo numero, una fraccion del trabajo, y **cero
riesgo de tocar la semantica de quien controla que cuenta**.

**B — hacerlo entero, en un commit.** Con el barrido comprobando las **dos**
cadenas en los 27 circuitos (§81) y la suite en 272 + 201 sin ignorados, un
desajuste de disposicion se caza al instante.

**A — coexistencia, solo si B no cabe.** Añadir una derivacion ancha junto a
la estrecha y migrar por partes, como los `_climb`.
⚠️ **Es la ultima opcion a proposito**: durante la migracion conviven **dos
formatos de identidad** en el sitio donde se decide quien es quien, que es
exactamente el terreno donde salieron §27 y §73.

### 85.7 B no es un commit: son dos, y hay un piloto dentro

Al comprobar el acoplamiento aparecio una descomposicion que **si** es
coherente, y que no es la que se descarto en §85.4.

| espacio de identidad | circuitos | atado por |
|---|---|---|
| **Gasto** (cuentas) | `audit`, `burn`, `claim`, `send`, `settlement` — **5** | `derive_public_id`, `native_nullifier` |
| **Custodios + gobernanza** | `freeze`, `governance`, `mint`, `mint_pending`, `recovery`, `threshold` ×3 — **8** | `build_custodian_set` → `derive_custodian_id` |

⚠️ **Custodios y gobernanza estan acoplados** y van juntos: el conjunto de
gobernanza se construye con `build_custodian_set`, la misma funcion y la
misma derivacion de hoja que el de custodios.

✅ **Gasto es independiente**: **ningun circuito mezcla los dos dominios**
—comprobado por `SPEND_KEY_DOMAIN` contra `CUSTODIAN_DOMAIN`—.

> La diferencia con lo descartado en §85.4 es que aquello era «un **circuito**
> cada vez», que parte un formato por la mitad. Esto es «un **espacio de
> identidad** cada vez», y cada uno es cerrado: los dos commits dejan el
> arbol verde.

### 85.8 El piloto: `circuit_settlement`

Es de los cinco de gasto **y su Air no lo ejecuta la capa** —solo lo
referencia `transfer.rs`, que no esta declarado en `lib.rs`—.

Se puede ensanchar **solo**, con una derivacion `_wide` usada unicamente
ahi, **sin introducir dos formatos en produccion**: que era la objecion a la
opcion A.

Establece el patron exacto —columnas, ranuras, grados, `build_trace`,
tests— para los otros cuatro. Y su coste ya esta medido: §86 lo cifra en
**−2,7 % de tamaño y −12,5 % de tiempo**.

**Orden**: piloto en `settlement` → los otros cuatro de gasto, la derivacion
compartida y `open_account`, en un commit → custodios y gobernanza, en otro.

### 85.9 Por que se para aqui

El analisis esta completo y no queda nada que investigar: la 15 pasa de
«hay que decidir» a «hay que escribirlo».

⚠️ Se para **a proposito**. Es el unico cambio de la sesion que toca
directamente el sitio donde se decide **quien controla que cuenta**, y los
dos fallos de solidez encontrados hoy —§73 y §74— salieron exactamente de
ahi.

> **Un analisis completo no caduca; una sesion larga si.** Empezar cansado
> un cambio de formato de identidad es la clase de decision que este
> documento registraria despues, y mejor registrarla antes.

### 85.6 Lo que este inventario ya ha evitado

Tres cosas, y ninguna se habria visto escribiendo codigo:

1. Que el plan «un circuito primero» era inviable (§85.4).
2. Que el coste era +15 y no +6.
3. Que el tamaño de prueba **si** cambia.

Las tres se dijeron mal antes de contarlas. Contarlas costo cuatro `grep`.

## 86. El coste de ensanchar: medido, y no es el que se esperaba

§85.2 conto que ensanchar `sk` a cuatro elementos cuesta **+3 columnas y +15
restricciones por circuito**. Faltaba saber que vale eso en tamaño y tiempo.

Se midio con **relleno** sobre `circuit_settlement` —cuyo Air la capa no
ejecuta— para no tocar la semantica de quien controla que cuenta.

### 86.1 Las seis medidas

| | prueba | generar |
|---|---|---|
| **49 columnas, 155 restricciones** | **40.645 B** ×3 | 111,3 / 112,3 / 111,8 ms |
| **52 columnas, 170 restricciones** | **39.538 B** ×3 | 97,6 / 98,0 / 97,8 ms |
| **delta** | **−1.107 B (−2,7 %)** | **−13,9 ms (−12,5 %)** |

Con mas columnas y mas restricciones, la prueba es **mas pequeña y mas
rapida**.

### 86.2 ⚠️ Un error mio, y del tipo peor

Con la primera pareja de medidas escribi que el instrumento estaba mal y que
el efecto era **ruido**, porque «el tamaño de una prueba STARK varia entre
ejecuciones».

**Es falso.** Tres ejecuciones de cada lado dieron el mismo byte exacto: el
tamaño es **determinista**.

> Atribui a ruido un efecto real **porque tenia el signo que no esperaba**.
> Es el mismo error que explicar un dato incomodo, y aqui es peor: el dato
> era **comodo** —favorecia la decision que ya queria tomar— y aun asi lo
> descarte por no encajar con mi modelo.

Se registra porque el sesgo no fue hacia la conclusion, sino hacia la
**expectativa**, y ese no se corrige queriendo.

### 86.3 Lo que la medida decide, y lo que no

**Decide**: el coste no es obstaculo. El plan B —hacerlo entero en un
commit— es viable, y no depende de entender por que.

**No decide** el porque. Tres columnas mas deberian dar una prueba mayor:
cada consulta abre tres valores adicionales. Que salga menor apunta a algo
estructural de winterfell —particionado del compromiso, numero de columnas
de composicion— y **cualquier explicacion que se diera aqui seria
especulacion sobre sus interioridades**.

### 86.4 La sospecha que si se comprobo

Un resultado favorable e inexplicado admite una lectura fea: que el circuito
con relleno **pruebe menos**, y sea mas pequeño porque hay menos que probar.
Las quince ranuras son copias redundantes y las tres columnas no las lee
nadie: no deberia pasar, pero «no deberia» es la palabra que este documento
lleva ochenta y seis secciones castigando.

Se corrio la suite del circuito con el relleno puesto: **16 de 16**,
incluidos los ocho negativos —`attacker_without_spend_key_cannot_transfer`,
`money_creation_is_rejected`, `forged_nullifier_is_rejected`…— y la prueba
por mutacion. Rechaza todo lo que debe rechazar.

**La sospecha queda descartada. La anomalia, no**: se abre como entrada 46.

### 86.5 Lo que se queda y lo que se va

El **relleno** se revierte: era un instrumento de una tarde.

El **instrumento de medida** se queda, con `#[ignore]`, porque mide el coste
real de este circuito y algun dia habra que volver a mirarlo. Es el mismo
criterio que `el_coste_de_agotar_el_espacio_de_claves` (§82.3): **un
instrumento no es una comprobacion, pero tampoco es basura**.

## 87. Fondos muertos: tres fuentes, dos obstaculos y ningun camino de vuelta

La entrada 12 decia que un pendiente no cobrado queda inmovilizado. Es
cierto y se queda corto en tres sitios.

### 87.1 No es una fuente, son TRES

| origen | quien lo provoca |
|---|---|
| El receptor **no cobra** | el receptor, o nadie |
| El destinatario **no existe** | el pagador, por error |
| El destinatario **esta CONGELADO** | ⚠️ **el propio sistema** |

Las dos primeras estan documentadas —la segunda tiene test propio,
`sending_to_a_nonexistent_recipient_loses_the_money`—.

⚠️ **La tercera es §29 y cambia la naturaleza del problema.** Enviar a una
cuenta congelada funciona; que ella cobre, no. En la via de un paso recibir
era **pasivo**; en la de dos fases **cobrar es una accion del receptor**, y
`circuit_claim` la rechaza si esta congelado.

> **Congelar una cuenta con pendientes pendientes los convierte en dinero
> muerto.** Y congelar es una accion legitima de los custodios: el sistema
> genera fondos irrecuperables **operando como debe**.

Eso es lo que hace el problema **sistemico** y no una suma de descuidos.

### 87.2 Obstaculo uno: la capa no tiene nocion de tiempo

Una expiracion necesita un plazo, y `lib.rs` lo dice sin rodeos:

> *«Es la rotacion de privilegios expresada por **uso**, no por tiempo:
> **esta capa no tiene nocion de tiempo**.»*

No es una omision: es la misma razon por la que la rotacion de custodios se
conto en **usos**. Cualquier caducidad tendra que expresarse en algo que la
capa **si** tenga —usos, altura de registro, o una operacion explicita— y
esa eleccion es de diseño, no de implementacion.

### 87.3 ⚠️ Obstaculo dos: el pendiente NO registra quien lo envio

```rust
pending_commitment(receiver_id, salt, amount)
```

Identidad del receptor, aleatorio, importe. **El emisor no aparece.**

> **La capa no puede devolver el dinero porque no sabe a quien.** No es que
> falte implementar la vuelta: es que **el dato no existe** en ninguna parte
> del estado.

Y no esta ahi por descuido. Que el pendiente no ate al emisor es lo que
impide correlacionar pagador y cobro: **es la privacidad del diseño**.

### 87.4 Lo que cualquier solucion tiene que probar

Ese es el nudo, y conviene enunciarlo antes de proponer nada:

1. **Que el plazo vencio**, en una magnitud que la capa tenga.
2. **Que el pendiente sigue sin cobrar** —el arbol lo dice: la hoja esta.
3. **Que quien recupera es quien pago**, sin que la capa aprenda el vinculo
   pagador↔pendiente que hoy no tiene.

El (3) es el dificil. Atar el emisor al compromiso lo resuelve y **destruye
la propiedad que el compromiso existe para dar**. Probarlo en cero
conocimiento —«conozco la clave del emisor de ESTE pendiente»— lo mantiene,
y exige un compromiso con dos identidades y un circuito que hoy no existe.

### 87.5 Lo que NO se propone aqui, y por que

Ningun mecanismo. Este documento **enuncia el problema y sus obstaculos**;
elegir entre caducidad por usos, por altura o por operacion explicita, y
entre atar al emisor o probarlo en cero conocimiento, es una decision de
diseño con consecuencias sobre la privacidad.

⚠️ Y una nota de honestidad sobre el alcance: **§29 tiene un sub-caso peor**.
Si un pendiente se vuelve muerto **porque los custodios congelaron al
receptor**, devolverlo al pagador es lo justo; pero un mecanismo que permita
recuperar pendientes de cuentas congeladas **puede usarse para vaciar a un
congelado** enviandole y recuperando. Eso hay que mirarlo cuando se diseñe,
no despues.

## 88. El mecanismo de reversion, propuesto y evaluado

§87 enuncio el problema de los fondos muertos sin proponer solucion. Aqui se
registra una **propuesta concreta** y su evaluacion contra el codigo.

### 88.1 La propuesta

Un circuito de **reclamo por reversion**: pasado un plazo, el emisor genera
una prueba alternativa que demuestra en cero conocimiento

- **(a)** que es el emisor original,
- **(b)** que el receptor nunca cobro,
- **(c)** que el plazo vencio,

sin revelar el saldo de nadie.

### 88.2 Lo que YA existe

**(b) funciona hoy.** Cobrar pone la hoja a cero; probar pertenencia del
compromiso en `pending_root` prueba que sigue sin cobrar. Es el inverso
exacto de lo que hace `circuit_claim`.

**(c) tiene las dos piezas.** `log.rs` lleva `seq: u64`, monotona por
construccion —`seq = entries.len()`—, encadenada en `chain_digest(seq, ...)`
y comprometida en `head()`; y el registro **verifica la secuencia entrada a
entrada**. La comparacion `altura > timeout` es la misma descomposicion de
Horner en segmentos que ya impone el tope de emision.

⚠️ **Cabo suelto**: `seq` **no es hoy entrada publica de ningun circuito**.
Meterla en la traza y atarla a algo que el verificador ya conozca es trabajo,
no obstaculo.

### 88.3 ⚠️ (a) no es un circuito que falta: es un dato que no existe

```rust
pending_commitment(receiver_id, salt, amount)
```

**No hay emisor.** No se puede probar ser el emisor de un compromiso que no
lo codifica: no hay nada contra lo que probar.

La propuesta exige por tanto **cambiar el formato del compromiso** para atar
`sender_id`. Eso invalida cualquier pendiente existente y toca `send`,
`claim`, `mint_to_pending`, `circuit_claim` —que lo reconstruye— y el aviso.

**Es la misma clase de cambio que la entrada 15**, con el coste ya medido en
§86: despreciable. Pero un formato es un formato.

⚠️ **La privacidad no se pierde**, y conviene ser exacto sobre donde cambia:

| | hoy | con `sender_id` atado |
|---|---|---|
| Frente a **terceros** | hash opaco | hash opaco — **sin cambio** |
| Al receptor, **por ISO** | ya lo sabe: el mensaje lleva `debtor_iban` | sin cambio |
| Al receptor, **via nativa** | ⚠️ **NO lo sabe**: `PendingNotice` es `{position, salt, amount}` | **pasaria a saberlo** |

O sea: **no es un no-cambio**. Por ISO da igual; por la via nativa el aviso
pasa a revelar el emisor al receptor. Es defendible —quien cobra suele saber
quien paga— pero hay que decirlo, no darlo por hecho.

### 88.4 Las cuatro piezas, y solo una es un circuito

| | pieza | naturaleza |
|---|---|---|
| 1 | El compromiso ata al emisor | **cambio de formato** |
| 2 | `seq` como entrada publica verificable | cambio de estado |
| 3 | Circuito de reversion | ~`circuit_claim` + un segmento |
| 4 | **Elegir el plazo** | ⚠️ **decision de politica** |

> «Totalmente implementable» es cierto en cuanto no hay obstaculo
> criptografico. Pero el hueco no era «falta ese circuito»: eran cuatro
> cosas, y **la cuarta no se implementa**.

### 88.5 La cuarta pieza: un plazo tiene victimas

Un timeout convierte «el receptor nunca cobra» en «el receptor no cobro **a
tiempo**». Y hay ausencias legitimas: sin conectividad, una clave en
recuperacion, un custodio en disputa.

⚠️ **No es un parametro: es una politica con perjudicados.** Registrarlo como
binario —«sin reversion» contra «reversion seca a plazo fijo»— seria empobrecer
la decision. Hay al menos dos patrones intermedios:

**Plazo extensible por el receptor.** Sin cobrar, publica una prueba barata
de «sigo vivo» —conocimiento de su clave sobre el compromiso— y extiende el
plazo. Convierte al perjudicado de «lento» en «ausente durante N
extensiones», que es un estandar mas defendible.
⚠️ **Coste**: otro circuito pequeño y una **mutacion de estado sobre
pendientes, que hoy son inmutables hasta el cobro**. Eso no es menor.

**Asimetria de plazos.** Reversion solo tras un plazo largo —meses, no
bloques—, con el razonamiento de que lo que se resuelve son fondos
**muertos** sistemicos, no fondos **lentos**. El parametro deja de ser un
compromiso de latencia y pasa a ser una **politica de abandono**, que es lo
que las jurisdicciones ya regulan para cuentas bancarias inactivas: hay
precedente institucional para elegirlo.

Ninguno elimina la decision con victimas: **el primero la desplaza y el
segundo la encarece.**

### 88.6 Y sigue en pie el sub-caso de §87.5

Un mecanismo que recupere pendientes de cuentas congeladas **puede usarse
para vaciar a un congelado**: enviarle y revertir. Con plazo extensible es
peor, porque el congelado **no puede extender** —extender exige probar
conocimiento de su clave sobre el compromiso, y eso es una accion que un
congelado quiza no deba poder hacer—.

**Hay que resolverlo al diseñar, no despues.**

## 89. Cuanto cuesta verificar, y en que se va el `apply`

`ESCALADO.md` dimensiona el sharding partiendo de **4 ms por prueba**, y los
presenta como medidos: *«Sobre lo medido (620 ms, 4 ms, 28,5 %)»*.

⚠️ **No lo estaban.** `metrics.rs` cronometraba `apply` —que verifica, muta
el arbol **y escribe a disco**—; la unica verificacion aislada que media era
la de la divulgacion de auditoria.

Y era el numero de mas peso del documento: la primera etapa de su cuello de
botella es «64 nucleos × 250/s», y los 250/s son 1/4 ms.

### 89.1 La medida

`metrics::tests::el_coste_de_verificar_una_prueba`, cinco ejecuciones
independientes en release:

| | |
|---|---|
| Muestras | 2,36 / 2,34 / 2,41 / 2,32 / 2,35 ms |
| **Media** | **2,35 ms**, dispersion 4 % |
| Pruebas/s por nucleo | **425** (el documento suponia 250) |

| | `ESCALADO.md` | medido |
|---|---|---|
| Verificar | 4 ms | **2,35 ms** |
| TPS/shard con margen del 50 % | 8.000 | **13.600** |
| **Shards para 498.000 TPS** | **64** | **37** |

El documento era conservador por un factor **1,7**.

### 89.2 ⚠️ Lo que de verdad no sabia nadie: en que se va el `apply`

| | |
|---|---|
| Verificar | **2,35 ms** — el **3,2 %** |
| Arbol y disco | **~70 ms** — el **96,8 %** |

> **Verificar la prueba es lo mas barato que hace la capa.** Todo el coste de
> aplicar esta en mutar el arbol y escribir.

Eso valida el `C4` de `ESCALADO.md` —epocas con `apply` por lotes— mejor de
lo que el propio documento argumenta: **el lote no ataca la verificacion,
ataca el 96,8 % que domina**. Amortizar una escritura y una actualizacion de
arbol entre miles de operaciones es exactamente donde esta el dinero.

### 89.3 ⚠️ El instrumento defectuoso sesgaba su propia medida

La primera lectura dio **2,80 ms**. El parche que añadio el instrumento
dejo un `#[test]` duplicado: `cargo test` lo registraba **dos veces** y los
corria **en paralelo**, dos hilos haciendo el mismo trabajo y compitiendo por
cache. Un **19 % mas lento**.

> **El defecto no solo duplicaba el test: sesgaba la medida hacia arriba.**

Es el segundo error de estructura de atributos del dia —el primero fue una
funcion postiza— y los dos vinieron de anclar en la linea `fn` olvidando lo
que la precede. El parche correctivo comprueba ahora que no quede ningun
`#[test]` duplicado ni delante de un comentario de documentacion.

⚠️ Y con la primera lectura se cito «2,80 ms» como si fuera exacta. Cinco
muestras despues, la dispersion es del 4 % y **la cifra honesta es un rango**,
no la ultima ejecucion que salio. Es la inversa del error de §86.2: alli el
dato era determinista y se llamo ruido; aqui varia y se cito como punto.

### 89.4 Lo que `ESCALADO.md` tiene que corregir

**No se integra todavia.** Este numero le obliga a cuatro cambios:

| | |
|---|---|
| §5 | 250/s → **425/s**, marcado como medido con fecha, `n` y rango |
| §6 | 64 → **37 shards**; rehacer la sensibilidad «× 4 peor» |
| §11 | Añadir el punto que faltaba **y cerrarlo** |
| §4 (C4) | Añadir que verificar es el 3,2 % del `apply` |

⚠️ Sobre §11: que un punto debil resulte **favorable no lo saca de la
lista**. Lo mueve de «incertidumbre» a «resuelto», y eso se escribe. Que el
valor real sea mejor no arregla que la etiqueta fuera falsa —es la
distincion de §76, donde el README decia «si pasa» y no pasaba—.

### 89.5 Lo que este numero NO dice

Mide **un nucleo de una maquina** con **estas `proof_options`**. Los 64
nucleos y el margen del 50 % siguen siendo supuestos de `ESCALADO.md`, no
medidas, y el coste del hash del arbol y el modelo de latencia de clientes
siguen siendo estimacion y modelo.

Se ha corregido **el primer factor**, que era el unico marcado como medido
sin estarlo.

## 90. Ensanchar la clave NO invalida ninguna cuenta

Primer paso de la entrada 15, y corrige una premisa que se venia arrastrando
desde §82.5.

### 90.1 Lo que se afirmaba

> *«Es un cambio de formato de identidad que **invalida cualquier cuenta
> existente**.»*

Lo repetian §85 y la entrada 15, y **entraba en la decision**: arreglar la 15
se justifico en parte con «no hay despliegue, asi que invalidar no cuesta».
Un argumento que solo vale mientras no haya usuarios.

### 90.2 Es falso, y lo dice un test

`derive_public_id_wide` y `native_nullifier_wide` se añadieron **junto a**
las estrechas, sin tocar ninguna firma. Y:

```rust
derive_public_id_wide(as_digest(sk)) == derive_public_id(sk)
```

Se cumple para todas las claves probadas. **La version ancha es una
generalizacion estricta**: rellenar con ceros devuelve exactamente la misma
identidad y el mismo nullifier.

| | se creia | es |
|---|---|---|
| Migracion | invalida toda cuenta existente | **conserva todas las identidades** |
| Lo que exige | reapertura forzosa | **rotacion gradual de claves** |
| Lo que gana quien no rota | — | nada: sigue con 64 bits |

### 90.3 El segundo test es el que hace que el primero signifique algo

`a_wide_key_is_not_its_first_element`: cambiar el **segundo** elemento **si**
cambia la identidad.

> Sin el, el primero pasaria igual si la version ancha **ignorara** los tres
> elementos nuevos —que es exactamente el fallo que tendria escrita mal—.

Uno fija que **generaliza**; el otro, que **usa** lo que se le da. Por
separado ninguno prueba nada util.

### 90.4 ⚠️ Conservar la identidad NO conserva la seguridad

Una clave rellenada con ceros sigue teniendo **64 bits de entropia** y sigue
cayendo en los 2^63 de §82.3.

> **Cambia el coste de MIGRAR, no el de ATACAR.**

Quien no rote su clave no gana nada. Lo que la version ancha permite es
**generar claves de 256 bits**; las viejas siguen valiendo lo que valian
hasta que se roten.

### 90.5 Lo que esto cambia del plan

- **La migracion deja de ser una ruptura.** No hay reapertura de cuentas: hay
  un formato que admite claves anchas y una rotacion que cada titular hace
  cuando puede.
- **Y el argumento «no hay despliegue» deja de hacer falta.** Se apoyaba en
  que invalidar era gratis por no haber usuarios; ahora no hay que invalidar.
  ⚠️ Conviene notarlo: **ese argumento habria envejecido mal** en cuanto
  hubiera un piloto, y se estuvo usando sin comprobar su premisa.
- **Y afecta a la entrada 28**: la limitacion que la cuarta revision tendria
  que publicar no es «el sistema no alcanza 128 bits», sino «las claves
  generadas antes de la rotacion tienen 64 bits». Es una frase distinta y
  mas fiel.

## 91. Tres mensajes de commit que afirmaron lo que no hicieron

Un fallo de **proceso**, no de codigo. Se registra porque el historial de
git es tan documento del proyecto como el README, y ahi la imagen fiel
fallo tres veces seguidas el mismo dia.

### 91.1 Lo que paso

| commit | contenia | decia |
|---|---|---|
| `a826f01` | el instrumento en `metrics.rs` | *«…and record what apply actually spends»* — **el registro entro despues** |
| `3b7e605` | las derivaciones anchas | *«…and correct the claim that widening invalidates accounts»* — **esa correccion no estaba** |
| `ecc60cb` | AUDITORIA y BACKLOG, 234 lineas | *«Add wide key derivations»* — **no añadia ninguna** |

El tercero salio de intentar arreglar el segundo: se ejecuto `git commit
--amend` sobre lo que **se suponia** que era `HEAD`, y entre medias habia
entrado otro commit. El resultado fueron **dos mensajes cruzados**, cada uno
describiendo lo que hizo el otro.

### 91.2 La causa es la misma de todo el dia

> **Operar sobre un estado que se supone en vez de sobre el que hay.**

Es lo mismo que hizo abortar tres parches por md5 desfasado, y lo mismo que
§76, §82.1 y §84.2 registran del README, de la entrada 15 y de los
preprints. Aqui llego a hacer daño **porque git no tiene assert**: con los
ficheros, el md5 para la operacion; con `--amend`, nada la para.

### 91.3 ⚠️ Y el segundo intento fue peor que el error

Enmendar sin comprobar `HEAD` **no arreglo un mensaje: estropeo otro**. Se
paso de un mensaje inexacto a dos cruzados.

> **Una correccion aplicada sin verificar el estado no es una correccion: es
> un segundo error con intencion de arreglar.**

### 91.4 Lo que se corrige y lo que NO

✅ `ecc60cb` → `2fb7438`: era `HEAD`, no lo habia visto nadie mas, se
enmendo.

⚠️ **`3b7e605` se queda como esta.** Esta publicado y su mensaje sobra en
una mitad. **No se reescribe**: hacerlo para que parezca que no hubo error
es exactamente lo contrario de lo que este documento defiende. Queda aqui
anotado, que es la unica reparacion coherente.

### 91.5 La regla que faltaba

Con los ficheros `.rs` **nunca** ha fallado un parche por estado
equivocado, y la razon es simple: ahi se pide `md5sum` **antes de generar
cada uno**, sin excepcion. Con la documentacion se dio por sabido el estado
—y fallaron tres— y con git ni siquiera se miro.

> **`git log --oneline -1` antes de cualquier `amend`, igual que `md5sum`
> antes de cualquier sustitucion.**

La disciplina existia; lo que fallo fue aplicarla solo donde resultaba
comodo. Es el patron de §59.2 —algo util aplicado a parte del trabajo sin
declarar a que parte— por **octava** vez en la sesion.

## 92. El piloto de la clave ancha: hecho

`circuit_settlement` opera con `sk` de **cuatro elementos**. Es el primero de
los cinco de gasto y el que la capa **no ejecuta** —solo lo referencia
`transfer.rs`, no declarado en `lib.rs`—, elegido para eso (§85.8).

### 92.1 Lo que cambio

| | antes | ahora |
|---|---|---|
| `COL_S_KEY` | 1 columna | **4** (25..29) |
| `TRACE_WIDTH` | 49 | **52** |
| `C_NULL_KEY` / `C_PK_INPUT` | 2 y 2 | **8 y 8** |
| `C_TRANSPORT` | 9 | **12** |
| `NUM_CONSTRAINTS` | 155 | **170** |
| Aserciones | 57 | **51** |

**18 tests del circuito en verde**, 274 y 201 sin regresion, 27 circuitos
limpios en las dos cadenas.

⚠️ Y **`no_constraint_is_vacuous` pasa**: las quince ranuras nuevas
**disparan todas**. No ocupan sitio: hacen trabajo. Sin esa prueba no
sabriamos distinguir una cosa de la otra.

### 92.2 Las aserciones que dejaron de tener sentido

El positivo fallo con `trace does not satisfy assertion main_trace(9, 552)`.
`get_assertions` fijaba a **cero** las ranuras `state[9..12]` en las dos
filas donde entra la clave. **Eran relleno; dejaron de serlo.**

Retirarlas **no afloja nada**:

> Las ranuras pasan de estar fijadas a **CERO** a estar fijadas a la **CLAVE
> DECLARADA** —por `C_PK_INPUT` y `C_NULL_KEY`, contra `COL_S_KEY`, que
> `C_TRANSPORT` mantiene constante y cuya `pk` derivada debe igualar
> `COL_S_ID` por `C_PK_CHECK`—. Es mas fuerte, no mas debil.

⚠️ La de la **fila 0** se queda: ahi 9..12 sigue siendo relleno de verdad,
porque la clave no entra hasta la fila 543. Lo caza un aserto del parche.

### 92.3 ⚠️ Tres descuidos, y los tres el mismo

El paso 2 dejo fuera un `state_a[8]`, una construccion de `SenderWitness` y
**`get_assertions` entera**.

> **Se cambio un formato y se reviso donde se ESCRIBE, no donde se
> COMPRUEBA.**

Es la variante de §91.5 —aplicar la disciplina solo donde resulta comodo—
en su forma mas concreta: al ensanchar una columna se buscan las
asignaciones y se olvidan las aserciones, que son las que dicen que esa
columna valia cero.

Ninguno llego a `main`: los dos primeros los caza el compilador y el tercero
lo cazo **el positivo, corrido primero y solo** (§66.2).

### 92.4 Y dos asertos mios que gritaron en falso

`spend_key: BaseElement` casaba tambien con el **parametro** de la derivacion
estrecha; `for i in 9..12` casaba tambien con la fila 0. Los dos abortaron el
parche por **su propia amplitud**, no por un fallo del codigo.

⚠️ Es preferible a lo contrario, pero conviene notarlo: **un aserto que grita
en falso enseña a ignorarlo**, que es exactamente lo que §78.4 dice de los 78
rojos permanentes. Los dos se afinaron en el sitio.

### 92.6 `circuit_burn`: el segundo, y lo que costo

Mismo patron, sobre el mas estrecho de los cinco.

| | antes | ahora |
|---|---|---|
| `TRACE_WIDTH` | 39 | **42** |
| `C_KEY_INPUT` | 2 | **8** |
| `C_TRANSPORT` | 7 | **10** |
| Aserciones | 33 | **33** — sin cambio |

⚠️ **Un paso menos que el piloto, y comprobado ANTES**: `circuit_burn` no
fija a cero las ranuras 9..12 en la fila de la clave —su unica asercion
sobre 9..12 es la de la fila 0, donde siguen siendo relleno—. Mirarlo antes
de escribir evito la ronda que el piloto costo (§92.2).

**11 tests en verde**, incluidos el positivo, el negativo del atacante y
`no_constraint_is_vacuous`.

### 92.7 ⚠️ CINCO rondas en el circuito mas simple, y siempre lo mismo

| ronda | lo que faltaba |
|---|---|
| 1 | un `state_a[8]` fuera del ancla |
| 2 | un `///` sobre un parametro —Rust no lo permite— y el `let key` del escenario |
| 3 | el **campo** `key` del struct y la **firma** de `run` |
| 4 | la clave del atacante |

En cada una se dijo «este es el ultimo» y no lo era.

> **La causa no fue el patron —que estaba probado— sino el ORDEN de
> aplicarlo**: se buscaron los USOS de memoria, uno a uno, en vez de cambiar
> los TIPOS y dejar que el compilador los enumere.

**La secuencia correcta, para los tres que quedan:**

1. Cambiar **todas las declaraciones**: campos de struct, firmas de funcion,
   parametros. Nada de usos todavia.
2. **Compilar.**
3. Arreglar lo que el compilador liste, que es **exhaustivo** y gratis.

Hacerlo al reves es hacer a mano —y peor— lo que la maquina hace sola. Se
hizo al reves cinco veces seguidas, y cada ronda consumio una compilacion
del interlocutor.

⚠️ Es la variante de §91.5 aplicada a uno mismo: **la disciplina existia
—dejar que la herramienta enumere— y se aplico solo donde resultaba
comodo.**

### 92.8 ⚠️ `main` estuvo roto, y las dos reglas que no se aplicaron

Ensanchar `circuit_burn` rompio `crates/zk-ssl/src/burn.rs`, que llama a su
`build_trace`. **Y se commiteo asi.**

**Regla no aplicada, primera.** El piloto no rompio la capa porque
`SettlementAir` **no lo ejecuta**: solo lo referencia `transfer.rs`, no
declarado en `lib.rs`. Eso esta escrito en §85.8, **por quien luego no lo
anticipo**.

> Se miro el circuito y no **quien lo usa**. Es la misma raiz que los cinco
> descuidos de §92.7: revisar donde se cambia y no donde repercute.

**Regla no aplicada, segunda, y es peor.** La salida de
`cargo test -p zk-ssl` **no imprimio ningun `test result`** —porque no
compilaba— y se dio por buena. El metodo de trabajo lo prohibe con esas
palabras: *«dar por bueno un commit sin ver la salida de los tests»*, y ya
dejo `main` roto una vez.

⚠️ Ademas paso **dos veces seguidas** que un `tail` corto escondio justo la
linea que decidia. Un filtro que recorta la salida es un filtro que puede
recortar la mala noticia: **`grep "test result"`, no `tail -6`.**

### 92.9 El arreglo fue una linea, y por que importa

La capa **no tuvo que cambiar**: se rellena la clave en el borde.

```rust
build_burn_trace([spend_key, ZERO, ZERO, ZERO], ...)
```

Cuadra porque §90 probo que rellenar **da la misma identidad**: la cuenta se
abrio con `derive_public_id(sk)` y el circuito computa
`derive_public_id_wide([sk,0,0,0])`, que es lo mismo.

> **Es la primera vez que la propiedad de §90 paga en codigo, y paga
> entero.** Sin ella habria habido que migrar `open_account` y toda la capa
> **con `main` roto**, que es la peor situacion posible para un cambio
> grande.

⚠️ **Y no gana seguridad**: siguen siendo 64 bits. Lo que falta es que
`open_account` acepte claves anchas y que se roten.

### 92.10 Lo que esto obliga para los circuitos que quedan

`send`, `claim` y `audit` **si los ejecuta la capa** —`two_phase.rs`,
`audit.rs`—. Ensanchar cualquiera de los tres **rompera su llamante**, igual
que `burn`.

**La secuencia, completada:**

1. Cambiar **todas las declaraciones** —campos, firmas— (§92.7).
2. **Compilar el crate del circuito.**
3. **Compilar `zk-ssl` TAMBIEN**, que es donde vive el llamante.
4. Rellenar en el borde de la capa, como en §92.9.
5. **Ver el `test result` de los DOS crates** antes de commitear.

El paso 3 y el 5 son los que faltaron.

### 92.11 `circuit_send`: el tercero, y §92.7 pagando

| | antes | ahora |
|---|---|---|
| `TRACE_WIDTH` | 49 | **52** |
| `C_KEY_INPUT` | 2 | **8** |
| `C_TRANSPORT` | 15 | **18** |

**17 tests del circuito en verde**, 274 y 201, 27 circuitos limpios.

⚠️ **Las cuatro declaraciones se localizaron ANTES de escribir** —`COL_KEY`,
la firma de `build_trace`, el campo de `Scenario` y la firma de `run`—. En
`burn` esas mismas cuatro costaron cinco rondas por buscarlas de una en una
(§92.7). Aqui: **una ronda para el circuito.**

### 92.12 ⚠️ La herramienta de §81 cazo un fallo real

Al primer intento, `check_constraint_layout.py` reporto **tres colisiones y
tres ranuras muertas**. La causa: desplazamientos codificados a mano
—`C_TRANSPORT + 7` y `+ 11`— que asumian un array de **7** columnas. Al
pasar a 10, las ranuras 7-9 se escribian **dos veces** y las 17-19 quedaban
**muertas**.

> Es exactamente el fallo de §66.2, y **lo detecto la ampliacion que se hizo
> el mismo dia**. Primera vez que esa herramienta paga en algo que la lectura
> no habria visto.

### 92.13 ⚠️ Rellenar en el borde vale para la CAPA y NO para el CLIENTE

§92.9 dejo escrito que los llamantes se arreglan rellenando la clave con
ceros. **Esa receta es correcta para la capa y falsa para el cliente**, y
estuvo a punto de aplicarse a los dos:

| llamante | que es | como se arregla |
|---|---|---|
| `two_phase.rs::send` | via **antigua**, `#[deprecated]` | relleno en el borde |
| `client.rs::prove_send` | **la via del cliente** | ⚠️ **la firma cambia a `Digest`** |

> Rellenar en `prove_send` habria hecho compilar todo y **dejado el trabajo
> sin efecto**: el cliente nunca podria usar una clave ancha, y los 256 bits
> del circuito no servirian a nadie. Es la version §32 del problema
> —completar unidades de trabajo sin alcanzar el objetivo— y la receta
> reciente la empujaba.

### 92.14 ⚠️ Y el hallazgo que reordena lo que queda

`a_whole_payment_without_giving_any_key_to_the_layer` —el test que §33 cita
como demostracion de la propiedad central— fallo con `NotTheAccountHolder`
al darle una clave ancha de verdad.

**Y el circuito tenia razon**: la cuenta se abre con
`open_and_fund(SK_ALICE)`, que deriva la identidad **estrecha**. Esa clave no
le corresponde.

> **Los 256 bits estan en `settlement`, `burn` y `send`, y ningun cliente
> puede usarlos.** `open_account` solo sabe crear cuentas con clave de 64,
> asi que la clave ancha existe en el circuito y **no es alcanzable desde la
> capa**.

⚠️ **Eso cambia el orden de lo que queda**: `open_account` sube por delante
de `claim` y `audit`. **Migrar mas circuitos no da un bit de seguridad mas**
mientras la puerta de entrada siga siendo estrecha.

Y lo destapo un fallo que parecia un descuido de test. La correccion —
rellenar con ceros— es la unica posible hoy, y **queda escrita en el propio
test junto con lo que no ejercita**, para que nadie lo lea como que el flujo
completo prueba los elementos nuevos. No los prueba.

## 93. La privacidad frente a terceros: cuatro superficies, medidas

El README afirma privacidad **frente a terceros**; el operador ve los
saldos y eso esta declarado. Se midio si lo primero es cierto.

⚠️ **Marco de falsacion**: los cuatro testigos afirman el comportamiento
**sano** —«leer exige autoridad», «los indices no son enumerables», «la hoja
no revela el saldo», «los indices no son predecibles»—. Afirmar el ataque
habria hecho pasar los tests por cualquier motivo (§66.2).

### 93.1 Superficie 1: el CONTRATO de `account_view`

```rust
pub fn account_view(&self, index: AccountIndex) -> Option<AccountView>
```

Toma un indice. **No recibe credencial. No comprueba nada.** Devuelve
`balance` y `nonce`. Medido: `Some(1000000)` de una cuenta ajena.

Y los indices son **enumerables**: `next_index += 1`, y barriendo 0..10
aparecen las 3 cuentas abiertas.

⚠️ **Esto es un hallazgo de CONTRATO, no de explotacion.** **No hay capa de
red en este repositorio**, asi que «cualquiera vuelca el ledger» seria una
afirmacion sobre un despliegue que no existe —una cifra rancia hacia el
drama, que §76 prohibe igual que hacia el aburrimiento—.

Lo cierto hoy, y solo esto:

> **El contrato de `account_view` no incluye autorizacion.** Exponerlo sin
> añadir control de acceso convierte una fuga hacia-el-operador —declarada
> en la cabecera de `client.rs`— en una fuga hacia-terceros, **que el README
> no declara**.

### 93.2 ⚠️ Superficie 2: la que SOBREVIVE a cerrar la API

`send_materials` entrega `sender_path`, y `sparse_tree::path_for` devuelve
`node(level, idx ^ 1)`: en el nivel 0, **la hoja del vecino**. Y

```rust
native_leaf(pk, saldo, nonce) = H(H(pk, saldo), nonce)
```

**sin salt** —verificado: `native_merge` pone la capacidad a cero, sin
dominio ni cegado, en las dos capas—.

**Medido: saldo del vecino recuperado en 10,84 s.** 743.100 centimos,
exacto.

| | |
|---|---|
| Hojas por segundo y nucleo | **68.549** (medido; se habia estimado 61.000) |
| Hojas probadas | 743.100 |

⚠️ **El coste no es un numero: es una curva sobre el rango de saldo
asumido**, que es un supuesto sobre la victima y no sobre la criptografia:

| rango asumido | coste, un nucleo |
|---|---|
| 0-10.000 EUR (2^20) | **2,4 min** |
| 0-1 M EUR (2^27) | 4,1 h |
| 0-100 M EUR (2^34) | 405 h |
| 64 bits uniformes | 8,3×10^7 años-nucleo |

> **La ultima fila salva y condena el diseño a la vez.** Si el saldo fuera
> uniforme en 64 bits el ataque seria inviable. **No lo es, y nunca lo es en
> un sistema de dinero**: la entropia del saldo es baja por definicion del
> dominio. Por eso el salt no es un lujo criptografico — **es la unica
> fuente de entropia posible**.

**Alcance, acotado**: el camino entrega 32 hermanos y **solo `siblings[0]`
es preimagen de hoja**. Los otros 31 son raices de subarbol y no son
diccionariables. Es **1 cuenta**, no log2(N).

**Regimen 1D, y no es un supuesto**: `accounts.rs` hace
`let nonce = BaseElement::ZERO` y sube de uno en uno por gasto. **El regimen
2D —donde el rango del nonce multiplicaria el coste— nunca existio.**

⚠️ **Y esta fuga sobrevive a cerrar `account_view`**: depende del formato de
hoja, no de la API. Tapiar la API es una tarde; esto no.

### 93.3 Superficie 3: el vecino es ELEGIBLE, no aleatorio

Medido: tres altas consecutivas dan indices `0, 1, 2`.

Con `next_index += 1`, **quien controla el momento de sus altas elige a
quien tiene por vecino de arbol** — y con dos altas, lo rodea. La fuga deja
de ser oportunista y pasa a ser dirigida.

### 93.4 La pregunta que decide el arreglo: **NO es clase entrada 15**

Un salt de hoja tiene que ser **secreto para el operador** —si no, no ciega—
y **recuperable por el cliente** —sin el no puede reconstruir su hoja—.

⚠️ **Y hoy el cliente no guarda nada.** `ClientState { public_id, balance,
nonce }`: los tres campos **los conoce la capa**, y `state_of()` los pide y
los recibe. Ese es el modelo entero.

> Un salt obliga a que el cliente **custodie estado que solo el tiene**. Eso
> no es un cambio de formato: **es un cambio de modelo de cliente**, y trae
> una decision con victimas —quien pierde el salt, pierde la cuenta—.

**Clase: entrada 15 MAS una decision de arquitectura que el proyecto no ha
tomado nunca.**

### 93.5 Y el obstaculo real no es el que se temia

Se temia que un salt dependiente del nonce rompiera el enlace entre la hoja
vieja y la nueva. **No lo rompe**: el circuito ya usa nonces distintos en
cada carril —`C_NONCE + 1` exige `nonce + 1` en el carril B— y **no liga las
hojas por campos compartidos**, sino porque ambas suben por el mismo camino
con los mismos hermanos. Un salt seria un campo mas, igual que el nonce.

⚠️ **Lo que si obstaculiza es la asimetria emisor/receptor**: `apply_send`
actualiza tambien la hoja del **receptor** —`C_NONCE + 2` y `+ 3`—, y con un
salt derivado de clave **el emisor no puede computarla**, porque no tiene la
clave del receptor. El pago exigiria que el receptor participe, y **el
diseño de dos fases existe precisamente para que no tenga que hacerlo**.

⚠️ **Ninguna salida a eso esta medida**, y no se registran las que se
discutieron: son ideas de diez minutos, y meterlas aqui seria lo que este
documento lleva noventa secciones castigando.

### 93.6 Lo que esto reordena

- **B12.1 —especificacion formal del AIR— no puede escribirse todavia.** El
  formato de hoja depende de una decision de arquitectura abierta:
  especificarlo ahora es especificarlo dos veces.
- **B11.2 —cifrar el aviso— tampoco**: §88.3 dice que el aviso puede tener
  que llevar `sender_id`, y cifrar antes de decidir el contenido fija el
  formato equivocado con AEAD encima.
- **B10.1 —cabeza firmada por epoca— sobrevive intacta**: no depende del
  formato de hoja, ni del aviso, ni de C3, ni de que haya testigos que
  comparen todavia.

### 93.7 Lo que NO se ha medido

1. Un despliegue con altas por lotes, donde atacante y victima quedarian
   contiguos **por construccion**.
2. Multinucleo y GPU: los 2,4 minutos son **un nucleo sin optimizar**. La
   cifra realista es peor, no mejor.
3. Si `send_materials` acabara expuesto en algun transporte. Hoy no hay.

## 94. Tres `native_leaf` con dominios de identidad distintos

Hallazgo aparte, encontrado al verificar cual usa la capa.

| | firma de la identidad |
|---|---|
| `circuit_settlement` (**la de produccion**) | `public_id: Digest` — **256 bits** |
| `compliance_circuit` | `account_id: BaseElement` — **64 bits** |
| `double_entry` | `id: BaseElement` — **64 bits** |

**Las tres tienen la MISMA estructura** —`H(H(id, saldo), nonce)`—. No
divergen en forma: divergen en **anchura del dominio**, porque
`as_digest(id)` es `[id, 0, 0, 0]`.

> Es **la entrada 15 replicada en la identidad** de dos circuitos
> secundarios. Y explica por que solo se corrigio `circuit_settlement`: fue
> el que se auditó.

⚠️ **Compartir nombre lo empeora**, porque invita a suponer que son la misma
funcion. Un lector que verifique `native_leaf` en un circuito y lo de por
bueno en los otros dos se equivoca, y nada en el codigo se lo advierte.

⚠️ **No se ha medido** si `compliance_circuit` o `double_entry` estan en
algun camino de produccion. `SettlementAir` no lo esta (§85.8); de estos dos
no se sabe.

## 95. Garantias con condicion implicita: la tercera vez

El README decia que la privacidad es «frente a terceros **que solo ven
pruebas**». Corregido hoy, porque §93 midio que esa condicion no se cumple.

### 95.1 No es una mentira: es una condicion sin verificar

La frase era **cierta bajo su condicion**. Lo que nadie comprobo es que la
condicion se diera: un tercero **no solo ve pruebas**. Ve `siblings[0]` en
los materiales del protocolo, y ve saldos si llama a `balance_of`.

### 95.2 ⚠️ Y es la TERCERA vez que aparece esta estructura

| | garantia | condicion implicita que fallaba |
|---|---|---|
| §76 | *«no puede reescribir el historial en secreto»* | …para quien **ya observo una cabeza anterior**, y nadie fuera del operador las observa |
| §84.2 | *«la clave de gasto no sale de la maquina del cliente»* | …**si la capa verificara la prueba**, y no la verificaba (§73) |
| §95 | *«privacidad frente a terceros que solo ven pruebas»* | …**si los terceros solo vieran pruebas**, y ven caminos y saldos |

Las tres son de la misma forma exacta:

> **Una propiedad del DISEÑO enunciada como propiedad del SISTEMA, con la
> condicion que las separa escrita o sobreentendida, y nunca verificada.**

Y las tres se descubrieron igual: **no buscandolas, sino escribiendo con
precision otra cosa y tropezando** —§76 midiendo cifras del README, §84.2
inventariando los preprints, §95 comprobando una errata del salt—.

### 95.3 El proyecto no tiene instrumento para esta clase

`check_constraint_layout.py` cruza indices. `buscar_vacias` prueba
mutaciones. Los tests comprueban comportamiento. **Ninguno lee una
afirmacion en prosa y verifica su condicion.**

⚠️ Y `B12.1` —la especificacion formal del AIR— **solo cubre los
circuitos**. Las garantias del README, de `PRINCIPIOS.md` y de los tres
preprints se siguen escribiendo sin contrato.

> El equivalente documental de B12.1 seria: **cada garantia publicada
> declara su condicion, y cada condicion tiene un test que la comprueba.**
> Las tres de la tabla habrian caido el dia que se escribieron.

No se propone como tarea: se enuncia porque es la leccion de las tres, y
sin enunciarla la cuarta llegara igual.

### 95.4 Lo que NO se ha hecho hoy

**No se ha tocado el codigo.** La mitigacion de la entrada 49 —exigir la
identidad publica para leer— es barata y **no se ha hecho**, por dos
razones:

1. **Su llave es la direccion de pago.** `public_id` es lo que se reparte
   para cobrar; un control cuya credencial la tiene todo el que te ha pagado
   **no es un control de acceso**, y no es revocable.
2. ⚠️ **Permitiria escribir una frase mas tranquilizadora sin dar la
   garantia que sugiere** — que es exactamente el defecto de §95.1, cometido
   a sabiendas.

Impide el **barrido masivo**, y eso es valor operativo real. Se hara **con
la entrada 50** —son la misma decision de arquitectura— y **declarando su
alcance en el mismo commit**: «impide la enumeracion; no impide la lectura
por quien conoce tu identidad de pago».

## 96. ⚠️ RECTIFICACION de §92.14: `open_account` va el ULTIMO

§92.14 —escrita hace una hora— concluyo que **`open_account` sube por
delante de `claim` y `audit`**, porque migrar mas circuitos no da un bit de
seguridad mientras la puerta de entrada sea estrecha.

**El razonamiento era correcto y la conclusion, al reves.**

### 96.1 Lo medido: siete operaciones del titular, una funcionaria

| operacion | circuito | ¿clave ancha de verdad? |
|---|---|---|
| `client::prove_send` | `circuit_send` ✅ | **funciona** |
| `two_phase::send` (antigua) | `circuit_send` ✅ | rellena — solo con ceros |
| `burn` | `circuit_burn` ✅ | rellena — solo con ceros |
| `client::prove_claim` | `circuit_claim` ❌ | **NO** |
| `two_phase::claim` | `circuit_claim` ❌ | **NO** |
| `audit` | `circuit_audit` ❌ | **NO** |
| `disclose_exact` | `circuit_audit` ❌ | **NO** |
| `prove_minimum` | `circuit_audit` ❌ | **NO** |

### 96.2 ⚠️ Lo que habria pasado

Una cuenta abierta con clave ancha **de verdad** —elementos no nulos— podria
**enviar** un pago y **nada mas**: no podria cobrarlo, ni quemar, ni
auditarse, porque esas vias derivan la identidad **estrecha** y no
coincidiria.

Y el sistema es de **dos fases**:

> **Enviar sin poder cobrar deja el dinero en un pendiente inmovilizado** —
> que es exactamente la entrada 12, provocada a proposito por la propia
> migracion.

> **Migrar `open_account` ahora abriria una puerta a un pasillo cerrado. Y
> peor: una puerta por la que se mete dinero y no se saca.**

### 96.3 Por que el error, y no fue de razonamiento

§92.14 midio bien —los 256 bits no son alcanzables— y de ahi salto a «hay
que abrir la puerta primero». **Lo que no se conto fue si habia salidas.**

Se propuso incluso el cambio concreto —firma a `Digest`— con el argumento
de que §90 evita romper cuentas existentes. Eso es cierto **y es
irrelevante**:

> El problema no era **lo que rompia**, sino que **lo que habilitaba no
> servia**. Un cambio puede ser seguro para lo que ya hay y aun asi crear
> algo inservible.

Lo paro una pregunta: *«¿cuantos circuitos faltan para que una clave ancha
sea usable de verdad?»* — es decir, **contar antes de proponer**, que es lo
que este documento lleva noventa y seis secciones repitiendo y lo que fallo
otra vez.

### 96.4 El orden correcto

**`claim` → `audit` → `open_account` y la capa.**

`claim` **primero de todos**, porque sin el una cuenta ancha no puede
recuperar su dinero. `open_account` **el ultimo**, porque es lo unico que no
puede ir antes que las salidas.

### 96.5 ⚠️ Riesgo que queda declarado

**Quien migre `open_account` sin leer esto crea cuentas que pierden fondos.**
No es un riesgo teorico: el cambio es de una linea, parece progreso, y §92.14
lo recomendaba.

Por eso se registra hoy y no cuando se retome.

### 92.15 `circuit_claim`: el cuarto, y el manual ya funciona

| | antes | ahora |
|---|---|---|
| `TRACE_WIDTH` | 48 | **51** |
| `C_KEY_INPUT` | 2 | **8** |
| `C_TRANSPORT` | 15 | **18** |

**Una ronda para el circuito**, dos para los llamantes. 274 y 201, 27
circuitos limpios **al primer intento**.

Las tres comprobaciones previas del manual, hechas antes de escribir:

1. **Las cuatro declaraciones** (§92.7).
2. **Las aserciones** (§92.6): la unica sobre `9..12` es la de la fila 0.
3. ⚠️ **Los desplazamientos a mano** (§92.12): `C_TRANSPORT + 7` y `+ 11`,
   corregidos **de entrada** en vez de esperar a que los cazara el barrido.

⚠️ **Ese tercero es reincidente y merece contarse**: el propio fichero ya
documentaba *«era `C_TRANSPORT + 7`, mismo solapamiento que send»* (entrada
36 / §50.7). Con la de `send` de hoy (§92.12), es la **tercera** vez que ese
desplazamiento concreto rompe algo. La primera se descubrio como fallo de
solidez; la segunda la cazo la herramienta; la tercera se vio leyendo.

### 92.16 ⚠️ CUATRO circuitos migrados, CERO bits de seguridad ganados

`settlement`, `burn`, `send` y `claim` verifican claves de 256 bits. Y en el
test del pago completo, **las dos claves —la del emisor y la del receptor—
van rellenadas con ceros**, porque sus cuentas se abrieron con
`open_account`, que deriva **estrecho**.

| | |
|---|---|
| Circuitos de gasto migrados | **4 de 5** |
| Bits de entropia que gana un titular hoy | **0** |

> **Un lector que vea «cuatro de cinco circuitos» concluira que la entrada 15
> esta casi hecha. La parte que da seguridad no ha empezado.**

No es un fracaso: es exactamente lo previsto en §96. **La seguridad entra
toda de golpe cuando se cierre la puerta, y ni un bit antes** — y por eso
`open_account` va la ultima, no la primera.

⚠️ Y es la razon para medir el progreso en **propiedades** y no en unidades
de trabajo: cuatro circuitos es un avance visible que no mueve la unica cifra
que importa. Es §32 otra vez, vista desde el lado del que va ganando.

### 92.17 `circuit_audit`: el quinto, y lo que el analisis previo encontro

`audit` no encajaba en el patron —`COL_KEY` en 13, ancho 24, **cero** sitios
`state[8] = spend_key`—, asi que se leyo antes de escribir, como §68 con
`mint_pending`.

**La razon resulto mas simple de lo que parecia:**

> **Es de UN SOLO CARRIL.** Cero apariciones de `LANE_B`. No absorbe la clave
> «de otra forma»: la absorbe **igual** —`state[4]=DOMINIO`,
> `state[8]=clave`— pero **una vez**, porque **audita en vez de transitar**:
> no hay estado viejo y nuevo que comparar.

| | antes | ahora |
|---|---|---|
| `TRACE_WIDTH` | 24 | **27** |
| `C_PK_INPUT` | 1 | **4** |
| `C_TRANSPORT` | 5 | **8** |
| `num_assertions` | 20 | **17** |

**+3 columnas y +6 ranuras: la mitad** que en los de dos carriles.

⚠️ **Y aqui SI habia aserciones que retirar** —tres, sobre `ROW_PK_START`—,
al reves que en `burn`, `send` y `claim`. Es el caso del piloto (§92.2), y
**se comprobo antes de escribir**, que alli costo una ronda. Son tres y no
seis porque hay un carril.

Las de la **fila 0** se quedan: ahi `9..12` sigue siendo relleno.

### 92.18 ⚠️ Sustituir o ampliar un import: la regla es CONTAR

Al migrar `audit` se sustituyo el import de la derivacion estrecha por la
ancha. Fallo: **algun test del circuito seguia usando la estrecha**.

Se corrigio ampliandolo —dejando las dos— y entonces salto el aviso
contrario: **`unused import`**, porque ya no quedaba ningun uso.

> Sustituir en un sitio y ampliar en otro no son dos reglas: **es una, y
> exige contar los usos que quedan.** Se hizo mal en las dos direcciones
> seguidas: sustituir donde habia que ampliar, y ampliar donde habia que
> sustituir.

El parche final lleva el aserto que lo distingue —`count("derive_public_id(")
== 0` antes de quitar el import— y que no se puso ninguna de las dos veces.

### 92.19 Los CINCO circuitos de gasto, migrados

`settlement`, `burn`, `send`, `claim`, `audit`. 274 y 201, cero avisos, 27
circuitos limpios.

⚠️ **Y siguen siendo CERO bits de seguridad ganados** (§92.16):
`open_account` deriva estrecho, asi que ningun titular puede tener una clave
ancha. Lo que queda —**la puerta**— es lo unico que mueve esa cifra, y la
mueve entera.

⚠️ Una nota sobre el orden que conviene conservar: **`audit` era el unico de
los cinco cuyo retraso no tenia consecuencia patrimonial.** No mueve dinero:
prueba `inferior <= saldo <= superior` sin tocar el arbol. Una cuenta que no
pudiera auditarse perderia la revelacion selectiva, no los fondos — a
diferencia de `claim`, cuya ausencia los inmovilizaba (§96.2).

## 97. La puerta: `open_account_wide`, y un pago de 256 bits medido

### 97.1 ⚠️ Primero se hizo mal: 115 llamadas, no «~22»

Se estimo el alcance de cambiar la firma en **«~22 llamadas»**. Medido:
`.open_account` + `_checked` **31**, `.send` **37**, `.claim` **25**,
`.burn` **14**, `.audit` + `disclose_exact` + `prove_minimum` **8** —
**115**. Al aplicarlo los errores pasaron de 15 a **85**.

> ⚠️ Es **§80.2 literal, escrito el mismo dia**: contar llamadas a una
> funcion no mide la superficie que depende de lo que esa funcion usa.
> Faltaban `send`, `claim`, `burn` y `audit`, cuyas firmas cambian con ella.

⚠️ **Y peor que el error de cuenta**: cuando el numero no cuadro, en vez de
parar se tiro de una **expresion regular** que hizo **18 sustituciones que
nunca se llegaron a ver**. Es justo lo que este metodo prohibe. Revertido
entero con `git checkout`.

### 97.2 El diseño correcto era el que se habia descartado

§85.5 rechazo la coexistencia por «dos formatos de identidad conviviendo».
**Con §90 medido, esa objecion es falsa:**

> `[sk,0,0,0]` da **la misma identidad** que `sk`. No conviven dos formatos:
> conviven dos **anchuras de entrada** al mismo. **El arbol no distingue**
> una cuenta abierta por una via de otra abierta por la otra — solo
> distingue **cuanta entropia** tiene su clave.

Se añadio sin tocar ninguna de las 115 llamadas: `open_account_wide`,
`open_and_fund_wide` y `wide_key`.

### 97.3 ✅ Un pago completo con clave de 256 bits, medido

`a_whole_payment_with_a_256_bit_key`, en verde: `open_account_wide` →
`send_materials` → `prove_send` → `apply_send` → `claim_materials` →
`prove_claim` → `apply_claim`, con los tres elementos extra **no nulos**.

**El camino nunca se habia recorrido.** 202 y 274, cero avisos, 27 circuitos
limpios.

> **La entrada 15 deja de valer cero bits.** Quien abra con
> `open_account_wide` tiene **256 bits**, y los 2,38 millones de años-nucleo
> de §82.3 dejan de aplicarle.

### 97.4 ⚠️ La deuda, declarada

**La migracion es opt-in.** Quien use `open_account` sigue con **64 bits**, y
son 115 llamadas y 158 usos de `open_and_fund` que lo hacen. La API queda con
**dos entradas de apertura**.

⚠️ **Para cerrar la 15 falta**: retirar o marcar la via estrecha —familia de
la entrada 32— y que las cuentas existentes **roten**.

## 98. Adopcion: la rotacion ya existia, y quien no puede usarla

§97.4 pedia dos cosas para cerrar la entrada 15. **Una ya estaba hecha.**

### 98.1 La via estrecha, marcada

`open_account` lleva `#[deprecated]`, como §65 hizo con las cinco vias
antiguas. **Cero churn** —es un aviso, no un cambio de firma— y **cero
avisos nuevos**, porque su unico llamante directo es `open_and_fund`, que ya
esta cubierto.

### 98.2 ✅ La rotacion no era trabajo pendiente: era un camino sin recorrer

```rust
pub fn recover(&self, auth, account_index, new_public_id: Digest)
pub fn apply_recovery_delegated(..., new_public_id: Digest)
```

Las dos toman **la identidad ya derivada** —un `Digest`— y **no comprueban
su formato**. Un titular con clave ancha siempre pudo rotar: deriva
`derive_public_id_wide(sk)` en su maquina y la pasa.

> **Lo que faltaba no era implementarlo: era comprobarlo**, que no es lo
> mismo. Es la distincion que este documento lleva noventa y ocho secciones
> haciendo, y §97.4 la habia perdido.

`a_narrow_account_can_rotate_to_a_256_bit_key`, en verde: abre estrecha,
rota a ancha, **y paga despues con los 256 bits**. Las cuentas que ya
existen **tienen salida**.

### 98.3 ⚠️ El testigo se escribio primero por la via equivocada

La primera version usaba `recover`, marcada `#[deprecated]` desde §65 porque
exige las claves de custodio **en el operador** — el fallo de la entrada 32.

> Habria demostrado la propiedad **por un camino que el proyecto quiere
> retirar**. Al retirarlo se habria perdido la evidencia de que las cuentas
> viejas tienen salida — justo lo que el testigo existe para probar.

Reescrito con `apply_recovery_delegated`. **Lo destapo el aviso del
compilador**, no la revision: sin `#[deprecated]` en `recover`, el testigo
se habria quedado asi.

### 98.4 ⚠️ Y la limitacion que sale de haber mirado

**`recover` exige DOS CUSTODIOS.** Rotar a clave ancha **no es una accion
soberana del titular**: necesita autorizacion de terceros.

| | |
|---|---|
| La clave de gasto | **nunca sale** de la maquina del titular |
| Cambiarla | **depende de dos custodios** |

> Un titular **no puede mejorar su propia seguridad por su cuenta**. Puede
> gastar sin permiso de nadie y no puede protegerse sin permiso de dos.

Es la **cuarta** condicion implicita de la familia de §95.2 —una propiedad
del diseño enunciada sin la condicion que la limita— y va como entrada 52,
no como nota: es una limitacion de **soberania**, que es la palabra del
titulo del proyecto.

## 99. ⚠️ RECTIFICACION de §93.5: el obstaculo del salt era otro

§93.5 concluyo que lo que impedia un salt derivado de la clave era la
**asimetria emisor/receptor**: *«con un salt derivado de clave, el emisor no
puede computar la hoja nueva del receptor»*.

**Es falso.**

### 99.1 El emisor nunca computa la hoja del receptor

`circuit_send` usa `COL_R_ID` **solo para construir el compromiso del
pendiente** (`C_PEND_IN`). **No hay `COL_R_BAL`, ni `r_nonce`, ni nada del
estado del receptor.**

> Y tiene todo el sentido: **el diseño de dos fases existe para eso**. El
> emisor no actualiza al receptor; deja un pendiente que el receptor cobra
> despues **con su propia clave**, por `circuit_claim`.

| quien | que hoja recompone | ¿tiene la clave? |
|---|---|---|
| Emisor (`send`) | **solo la suya** | ✅ |
| Receptor (`claim`) | **solo la suya** | ✅ |

⚠️ **De donde salio el error**: se leyeron `C_NONCE + 2` y `+ 3` en
`circuit_settlement` —el circuito de UN paso, **que la capa no ejecuta**— y
se aplico la conclusion a la via de dos fases, **que es la que corre**. Un
dato correcto sobre el circuito equivocado.

### 99.2 Y §93.4 tambien quedo desfasada, por §97

§93.4 dijo que un salt exige que el cliente **custodie estado**, porque
`ClientState` lo pide todo a la capa. Desde §97, `prove_send` y `prove_claim`
reciben `spend_key: Digest`: **el cliente ya pasa su clave de 256 bits**, asi
que `salt = H(DOMINIO, sk, nonce)` seria derivable **sin almacen nuevo** y
con 256 bits de entropia.

### 99.3 ⚠️ El obstaculo REAL: la capa escribe hojas sin conocer el secreto

| operacion | quien compone la hoja | ¿tiene `sk`? |
|---|---|---|
| `open_account` | la capa | ❌ **no la guarda** |
| `mint` | la capa + custodios | ❌ **no** |
| `freeze`, `recover` | custodios | ❌ **no** |
| `send`, `claim`, `burn` | el titular | ✅ |

`circuit_mint` usa `COL_KEY_A` y `COL_KEY_B` —las claves de los
**custodios**— y recompone `native_leaf(public_id, balance, nonce)` **sin la
del titular**.

> **Tres operaciones privilegiadas escriben la hoja de un titular sin poder
> derivar su salt.** Emitir, congelar y recuperar son legitimas y **no pasan
> por el titular**. Si el salt viene de `sk`, esas hojas serian
> incomputables — o habria que darle el salt a la capa, y entonces no ciega
> nada.

**Eso descarta las derivaciones de `sk`**, que era la unica familia de
soluciones que parecia viable.

### 99.4 Lo que NO se registra

Se pensaron tres salidas —salt en el estado, salt que solo cambia en
operaciones del titular, cegar solo el saldo— y **ninguna esta medida**. La
tercera puede que ni tenga sentido criptografico.

⚠️ **No se registran**, por lo mismo que §93.5 no registro las suyas: son
ideas de dos minutos, y meterlas aqui las convertiria en punto de partida
del proximo que lea. **Lo solido es que el obstaculo conocido era falso y el
real es otro.**

### 99.5 Y una leccion sobre el propio registro

§93.5 se escribio con el rigor de esta casa —contra el codigo, con la
comprobacion hecha— y **estaba mal**, porque se leyo el circuito que no
corre.

> **Un dato correcto sobre el objeto equivocado es indistinguible de un dato
> correcto**, y sobrevive a la revision precisamente porque es verificable.
> Lo unico que lo caza es **volver a preguntar de que objeto hablamos**.

Es la cuarta vez que un hallazgo de este documento se corrige a si mismo
—§82.5 en §90, §92.14 en §96, §93.5 aqui— y las cuatro se cazaron **al
intentar usar la conclusion**, no al releerla.

## 100. La cuarta revision de los preprints: los cuatro frentes escritos

§84 inventario cuatro frentes y once pasajes. Escritos hoy en `PAPER.md`,
`PAPER_EN.md` y `QUESTIONS.md`.

### 100.1 Las dos notas de correccion

**A y B no eran erratas: eran propiedades del DISEÑO enunciadas como
propiedades del SISTEMA** (§84.2). Las notas lo dicen sin rodeos:

> *«…la capa **no verificaba las pruebas** de la via de pago antes de aplicar
> la transicion, de modo que la clave no hacia falta para mover fondos
> ajenos.»*

> *«…el circuito de cobro **no ataba el compromiso a la identidad de quien
> cobra**: cualquiera con el aviso podia reclamarlo.»*

§84.3 decidio anotarlo con el precedente de `PRINCIPIOS.md` §8. ⚠️ Aquel era
una **prevision optimista** y estos son **garantias de seguridad**: la
decision se reconfirmo con la frase concreta delante, no en abstracto.

### 100.2 ⚠️ El frente C era peor de lo que §84 creia

§84 lo planteo como un analisis **que falta**. Al mirarlo resulto que
`PAPER.md` **§8.2 ya existe** —«El campo Goldilocks es demasiado estrecho
para identidades»— y **publica una correccion incompleta como completa**:

> *«La correccion consiste en emplear digests completos de cuatro elementos
> (256 bits).»*

Eso arregla la **identidad** y no la **clave** (§82.2, §90.4). No faltaba un
analisis: **habia una afirmacion publicada que se queda corta**, y es
distinto — en A y B el diseño era correcto; **en C la afirmacion era
incorrecta**.

La nota nueva aplica el criterio de §8.3 —los ~128 bits— al espacio de
claves, da los 2^63 medidos y declara que **la migracion es opt-in**.

### 100.3 ⚠️ Y `QUESTIONS.md` se quedo fuera del primer parche

§84.5 listaba **seis** sitios para la unidad y el parche cubrio **cuatro**:
los dos de `QUESTIONS.md` se olvidaron. Se detecto al barrer el tercer
documento **antes de registrar**, no despues.

> El inventario era correcto y **el parche no lo cubrio entero**. Es el
> patron de §59.2 —algo aplicado a parte del trabajo sin declarar a que
> parte— en el commit que corrige material publicado.

### 100.4 Lo que NO cierra la entrada 28

⚠️ **No se ha subido nada a Zenodo, y no debe subirse todavia.**

- **La entrada 16 sigue abierta**: los tres preprints se citan entre si por
  versiones antiguas. Subir una cuarta revision con referencias cruzadas
  rotas **crearia el mismo problema otra vez**.
- Los cuatro frentes estan escritos; **la revision no esta publicada**.

## 101. ⚠️ §84 inventario los preprints EQUIVOCADOS

### 101.1 Lo que paso

§84 localizo once pasajes en `PAPER.md` y `QUESTIONS.md` de la raiz. **Esos
no son los preprints publicados.** Los del DOI estan en `doc/preprints/`:

| | raiz | publicado |
|---|---|---|
| Lineas | `PAPER.md` **978** | `ZK-SSL-preprint.md` **448** |
| §8.2 | «El campo Goldilocks es demasiado estrecho para identidades» | «Properties proven on a model that is not executed» |
| «la clave de gasto no sale» | si | **no aparece** |

Las cuatro notas escritas el 01-08 estan en **material interno**. Corrigen
afirmaciones reales y esos ficheros tambien deben ser fieles, pero **la
entrada 28 no estaba hecha**, y el commit que las llevaba se llama *«Fourth
revision of the preprints»* sin haber tocado ningun preprint.

> Es **§99.5 por segunda vez en la misma sesion**: un dato correcto sobre el
> objeto equivocado es indistinguible de un dato correcto.

⚠️ Y otra vez lo cazo **intentar usar la conclusion** —hacer la entrada 16—
no releer §84. Van **seis** autocorrecciones hoy y las seis igual.

### 101.2 La entrada 16 tenia dos citas mas de las que decia

Siete DOI apuntaban a primeras revisiones. **Cinco cruzadas** —lo
inventariado— **y dos autocitas**: `ZK-SSL-preprint.md` y
`ZK-SSL-policy-note.md` **se citaban a si mismos** con su DOI antiguo. Eso
no estaba en la entrada.

### 101.3 ⚠️ Lo que aparecio al leer `ZK-SSL-residual-trust.md`

Su tabla §4.1 enumera lo que el operador puede hacer, y **dos filas
enunciaban garantias que no se cumplian**:

| fila | decia | la realidad |
|---|---|---|
| *Create value outside rules* | «Constrained by proof verification» | §74: **se creo dinero fuera del tope** |
| *Spend from an account without key* | «Constrained **if** spending proofs require client-side keys» | §73: **la capa no verificaba** |

> **La tabla ya tenia el formato correcto.** Distingue «Yes», «Constrained»,
> «Not bounded», y usa «constrained **if** X» cuando la garantia es
> condicional. **Lo que fallo no fue el diseño del documento: fue verificar
> que X se cumpliera.** §95.2 dentro del papel que lo padece.

Y **falta una fila entera**: la tabla no contempla al **tercero**. §4.4 cerro
la fuga hacia el pagador; la de §93 va hacia un **vecino que no ha pagado
nada**, y ese documento se compromete —§24— a que *«un paper cuya
contribucion es nombrar la confianza residual queda falsado por la confianza
que no supo nombrar»*.

### 101.4 §4.7: la primera seccion de ese paper sin ✅

Se añadio con los tres residuales: las dos condiciones incumplidas y la
privacidad frente a terceros, con los **10,84 s**, la curva sobre el rango de
saldo, el alcance de **una cuenta** y el vecino **elegible**.

⚠️ **No termina en checkmark**, y es lo correcto: §99 descarto la familia de
soluciones. Un documento cuya tesis es nombrar lo que falta, con todos sus
residuales cerrados, es sospechoso.

### 101.5 Lo hecho, y lo que sigue sin hacerse

✅ Siete DOI · §4.1 dos filas · §4.5 la unidad, **tercera correccion de esa
cifra** · §4.7 · frente A en `policy-note`, redactado para su lector
institucional · `README` del directorio, de «pendiente conocido» a «escrita,
sin depositar».

⚠️ **NO se ha depositado nada en Zenodo.** Los DOI siguen apuntando a las
terceras revisiones, **que son las que un lector recibe hoy**.

## 102. La entrada 15, cerrada: la adopcion es opt-in por diseño

Se intento migrar la suite entera a claves anchas cambiando **una linea**:
`open_and_fund` llamando a `open_account_wide(wide_key(sk))`, que habria
convertido **159 tests** de golpe.

### 102.1 Lo que paso, medido

Compilo limpio y **fallaron 62** —59 reales, mas los 3 testigos de
privacidad que deben fallar—. Agrupados por mensaje:

| | |
|---|---|
| `NotTheAccountHolder` | ~53 |
| `assertion left == right` | 6 |

⚠️ **Ninguno era del camino ancho.** Los tests abren cuenta ancha y luego
pasan una clave **estrecha** a `send`, `claim` o `burn`, cada uno con su
literal. Dos patrones, 59 ediciones a mano.

### 102.2 Por que se revirtio

**1. No gana ni un bit.** §90 garantiza que una clave rellenada da la misma
identidad, asi que `open_and_fund` estrecha es correcta. Y la capacidad de
256 bits **ya esta probada de punta a punta** —§97.3 el pago completo, §98.2
la rotacion—. Migrar los 159 cambia **con que claves prueban los tests**, no
lo que el sistema puede hacer.

**2. Son 59 ediciones a mano en dos patrones**, y este mismo dia se
demostro dos veces que ahi el error es alto: el `regex` que hizo 18
sustituciones sin verse (§97.1) y las cinco rondas de `circuit_burn`
(§92.7).

**3. Y hay un argumento mejor que los dos anteriores.** Que 159 tests usen
la via estrecha **no es deuda: es cobertura**.

> La via estrecha **sigue existiendo y sigue siendo llamable**. Si toda la
> suite pasa a ancha, **la estrecha queda sin un solo test** y permanece en
> el codigo. Migrar la cobertura de lo que sigue vivo es empeorar, no
> mejorar.

### 102.3 Lo que cierra la entrada

| | |
|---|---|
| **Capacidad** | ✅ cinco circuitos, la puerta, el pago completo, la rotacion |
| **Adopcion** | **decision de quien abre cuenta**, no del proyecto |

§97.4 ya lo decia y era exacto: **la migracion es opt-in**. La entrada 15 se
cierra **declarandolo**, no migrando 159 tests.

⚠️ **Y cuando deje de ser opt-in tendra otro nombre**: retirar la via
estrecha es la **entrada 32**, y entonces los 159 tests habra que migrarlos
—pero por esa razon y con ese criterio, no por esta—.

## 103. B10.1 no es independiente: no hay firma en el proyecto

`CONFIANZA_RESIDUAL.md` clasifica B10.1 —cabeza firmada por epoca— como
«formato + componente», con la nota: *«el log ya tiene `seq` y
`chain_digest`; publicar es aditivo»*. Y §93.6 la llamo **la unica pieza sin
dependencias**.

### 103.1 Cierto para publicar, falso para firmar

**No existe ninguna primitiva de firma en el proyecto.** Verificado: cero
coincidencias de `ed25519`, `dalek`, `signature` o `sign(` en todo el arbol,
incluidos los `Cargo.toml`.

Y B10.1 dice literalmente que el operador **firma** y publica. Sin firma, la
**prueba de fraude portable** —el 90 % del valor de la propuesta— no existe:
cualquiera podria fabricar una cabeza.

⚠️ §93.6 miro `seq` y `chain_digest`, que si estan, y **no miro la firma**.
Es la misma clase de error que §84 inventariando los ficheros equivocados:
la comprobacion fue correcta sobre una parte del objeto.

### 103.2 Y elegir el esquema es una decision de TESIS

`ed25519` es la opcion obvia —madura, rapida, estandar— **y no es
post-cuantica**.

> El proyecto eligio STARK **precisamente** por transparencia y resistencia
> post-cuantica. Firmar las cabezas —que son el compromiso de **todo el
> historial**— con una curva eliptica meteria el supuesto que el nucleo
> rechazo por diseño.

No es «añadir una dependencia»: es decidir si la propiedad post-cuantica
aplica al historial o solo a las pruebas.

### 103.3 La propuesta ya tenia la respuesta y no la conecto

`CONFIANZA_RESIDUAL.md` lista **B15 — cabezas XMSS**, con `seq` como indice.
**B10.1 depende de B15, y ninguna de las dos lo declara.**

⚠️ Y la observacion de B15 es buena, mas de lo que ella misma dice: XMSS
tiene **estado** —un indice que no puede reusarse— y reusarlo **filtra la
clave**. En este uso concreto ese modo de fallo es una **ventaja**:

> `seq` es monotona por construccion, asi que reusar indice significa **dos
> cabezas con el mismo `seq`** — que es exactamente la vista dividida que se
> persigue. **El modo de fallo del esquema y el fraude perseguido son el
> mismo evento.**

### 103.4 Lo que separa DETECTABLE de OPONIBLE

| | sin firma | con firma |
|---|---|---|
| Dos testigos comparan cabezas del mismo `seq` | **detectan** la inconsistencia | igual |
| Probar ante un tercero **quien** la emitio | ❌ imposible | ✅ prueba portable |

**La primera mitad se puede hacer hoy** y tiene valor propio: una cabeza
publicada permite a cualquiera comprobar que su vista coincide con la de otro.
Lo que no da es oponibilidad.

**Se hace esa mitad, y se declara cual es** — en vez de llamarlo B10.1 y
dejar la firma implicita.

## 104. La cabeza de epoca: la mitad de B10.1 que no necesita firma

### 104.1 Lo que se construyo

`EpochHead` con cinco campos —`seq`, las tres raices y `chain_digest`— y un
`digest()` para compararlas de un vistazo. `SovereignLayer::epoch_head()` la
produce.

⚠️ **Todo esto ya existia. Lo que faltaba era exponerlo junto.** El README
afirma que el operador «no puede reescribir el historial en secreto», y §76
establecio que esa garantia **solo vale para quien ya observo una cabeza
anterior** — y hoy nadie fuera del operador observa cabezas.

**Dos testigos**, y el segundo importa mas:

- `two_divergent_views_produce_different_heads`: dos vistas con historias
  distintas dan cabezas distintas. **La vista dividida es detectable.**
- `a_head_does_not_say_who_issued_it`: **fabrica una cabeza a mano** con
  valores inventados y comprueba que es del mismo tipo que una legitima.

> El segundo test documenta **lo que el sistema NO hace**, en codigo
> ejecutable. Y el dia que alguien añada firma, **ese test fallara** — que es
> correcto: cuando la ausencia deje de ser cierta, el test que la afirma
> tiene que romperse.

### 104.2 ⚠️ Media pieza de B10.2 ya estaba construida

`TransitionLog::first_divergence` **ya existe**: compara dos registros y
devuelve la entrada exacta en que divergen.

`CONFIANZA_RESIDUAL.md` lista B10.2 —«comparacion entre testigos»— como
pendiente y la llama «la pieza que CT infradesplego». **La mitad
algoritmica ya estaba en el codigo.**

> Es la **tercera** vez hoy que una propuesta externa pide algo que el
> proyecto ya tiene: la rotacion de §98.2, `seq` monotona de §88.2, y esto.
> ⚠️ **Y las tres se descubrieron al ir a implementarlas**, no al leer la
> propuesta.

### 104.3 El campo que falta, y por que no se pone vacio

`CONFIANZA_RESIDUAL.md` §2.1 incluye `hash_verificador_vigente` en la cabeza,
con el mejor argumento de esa propuesta:

> *«quien puede actualizar el verificador es la **raiz de confianza real** del
> sistema y nadie lo ve»*

**Cambiar el verificador cambia que es una transicion valida** — mas poderoso
que cualquier operacion del sistema.

⚠️ **No se puede rellenar hoy**: el proyecto **no tiene noción de «reglas
vigentes»**. `OpKind` dice que circuito usar, no que version de las reglas
estaba activa. Un campo vacio seria peor que su ausencia: **una cabeza que
dice incluirlo y no lo hace.**

Va **declarado en el propio tipo**, con su motivo, y abre la entrada 54.

### 104.4 Lo que esto NO cierra

| | |
|---|---|
| Vista dividida | **detectable**, no cerrada |
| Prueba de fraude | ❌ **no oponible** — sin firma (entrada 53) |
| Testigos que comparen | ❌ **no existen**: publicar es una funcion; recoger y comparar es operacion |

`CONFIANZA_RESIDUAL.md` §8.1 lo dice sin adornos: *la independencia de los
testigos es un supuesto social, no criptografico*. Y §6 recuerda que
Certificate Transparency tuvo el patron funcionando y su pieza de comparacion
**infradesplegada durante años**.

## 105. B12.1: una especificacion escrita, y por que no se escriben las 26

`SECURITY.md` §3.1 llama a la especificacion formal del AIR **la carencia de
prioridad mas alta**. Se escribio una —`doc/air/circuit_burn.md`, el circuito
mas pequeño de los cinco de gasto— para ver que produce.

### 105.1 Lo que NO encontro

**Ningun fallo.** Las 28 restricciones de `circuit_burn` estan completas.

### 105.2 Lo que si produjo, y no existia

**Uno. La asimetria de los dos lanes.** El lane A lleva la hoja **vieja**, el
B la **nueva**; en la fase de identidad los dos reciben la **misma clave** y
**solo el A se compara** con `COL_ACC_ID`.

Es correcto. Y la justificacion —«computan el mismo digest porque reciben la
misma entrada y la misma capacidad»— **se deducia cruzando las lineas 565,
606 y 615**, tres bloques separados por cincuenta lineas. **Ninguno lo
decia.**

**Dos. Dos garantias por CONSECUENCIA, no por restriccion.**

| | protege porque… | frágil si… |
|---|---|---|
| `C_PK_CHECK` (§4.2) | los dos lanes computan lo mismo | alguien cambia uno y no el otro |
| Camino de congelados (§4.5) | la raiz publicada no cuadraria | el arbol degenera |

> **Las dos dependen de que algo mas no cambie, y nada avisa si cambia.**
> §72 —un fallo de solidez— fue exactamente esa forma.

**Tres. Una correccion del propio documento mientras se escribia.** §4.5
empezo diciendo «nada verifica que el camino sea del mismo indice, y no lo he
podido descartar». Es falso: **la raiz lo verifica implicitamente**. §99.5
aplicado al documento en la hora en que se escribio.

### 105.3 ⚠️ Por que NO se escriben las otras 26

**No es por el esfuerzo**, aunque sea real: `circuit_burn` es el mas pequeño y
llevo una hora larga; varios de los 26 tienen el doble de restricciones.

Es por esto:

> **Una especificacion escrita por quien escribio el circuito hereda sus
> puntos ciegos.**

Las tres cosas que salieron fueron **razonamientos que hubo que reconstruir**
—por que basta comprobar un lane, por que la raiz cierra el camino de
congelados—. Un tercero no los reconstruye: **los cuestiona**. Y esa
diferencia es justo lo que separa una especificacion util de una que
documenta lo que su autor ya creia.

⚠️ **Y hay un riesgo peor que no escribirlas**: una especificacion completa,
firmada por el autor, **parece un contrato** — y un auditor que la reciba
puede auditarla **en vez de** auditar el codigo. `SECURITY.md` §3.1 pide un
contrato «contra el que contrastar»; un contrato escrito por la misma mano
que el codigo no contrasta nada.

### 105.4 Lo que se hace en su lugar

`doc/air/circuit_burn.md` queda como **muestra del formato**, no como primera
de veintisiete. Su valor es doble:

1. **Enseña que el formato funciona**: la seccion «que NO se restringe» es la
   unica que no se puede extraer del codigo, y es la que produjo las tres
   cosas de §105.2.
2. **Da al auditor un punto de partida**: el formato, un ejemplo completo, y
   la advertencia de que lo escribio el autor.

**La entrada 48 (B12.1) se mantiene abierta**, y su forma correcta es: **que
la especificacion se escriba con la auditoria, no antes de ella** — entrada 7.

⚠️ Se registra que se intento la via de generarla automaticamente y **no
sirve**: dos barridos —columnas sin usar, columnas en un solo carril—
aprobaron `circuit_burn` sin ver nada, porque §72 fue una restriccion **bien
formada sobre el objeto equivocado**, y eso ninguna herramienta que cuente
apariciones lo detecta.

## 106. DECISION: todo post-cuantico en produccion, XMSS para las firmas

Tomada el 01-08-2026 al plantearse la firma de las cabezas de epoca
(entrada 53). No es una preferencia: **es la tesis del proyecto aplicada a lo
que se añada a partir de ahora.**

### 106.1 El enunciado, con su matiz

> **Todo el camino de produccion es post-cuantico.** Los backends comparados
> —BLS12-381, Groth16, PLONK, Halo2, y `crates/ceremony`— **se conservan como
> evidencia de por que se eligio STARK**, no como alternativa desplegable.

⚠️ El matiz importa: retirarlos **destruiria la aportacion medible del
proyecto**, que es precisamente la comparacion empirica de cinco sistemas
sobre el mismo circuito. Estan ahi para mostrar el coste de las otras
opciones, incluida su dependencia de curvas.

### 106.2 Verificado, no supuesto

`crates/zk-ssl` y `crates/stark-experiment` —el camino de produccion— **no
tienen ninguna dependencia de curva eliptica**. Comprobado sobre sus
`Cargo.toml`: cero `ark-*`, `bls12`, `curve25519`, `secp` o `ed25519`.

Lo que sostiene el sistema hoy:

| pieza | supuesto |
|---|---|
| STARK / FRI | resistencia de la funcion hash |
| Rescue | idem |
| Arbol de Merkle | idem |

**Una sola familia de supuestos.** Y esa es la propiedad que la decision
protege.

### 106.3 XMSS, y el criterio que lo elige

No se elige por ser «el mas post-cuantico». Se elige porque **es el unico que
no añade una familia de supuestos nueva**:

| | supuestos | coste |
|---|---|---|
| **XMSS** | **resistencia de la funcion hash** — los mismos que ya sostienen los STARK | tiene **estado** |
| SPHINCS+ | hash tambien, sin estado | firmas de 8-17 KB |
| ML-DSA | **reticulos** — hipotesis nueva | estandarizado, rapido |
| ~~ed25519~~ | ❌ **no post-cuantico** | descartado por la decision |

> **Elegir XMSS no añade nada que no estuviera ya asumido.** Elegir ML-DSA
> si, y **un sistema con dos familias de supuestos es tan fuerte como la mas
> debil de las dos**: añadir reticulos junto a hashes no suma seguridad, suma
> superficie.

### 106.4 ⚠️ La condicion de XMSS, y por que aqui es una ventaja

XMSS es un esquema **con estado**: cada firma consume un indice del arbol, y
**reusar un indice filtra la clave privada**. En la mayoria de usos eso es un
inconveniente operativo serio.

Aqui no:

> `seq` es **monotona por construccion** —`seq = entries.len()`—, asi que
> usarla como indice significa que **reusar un indice es emitir dos cabezas
> con el mismo `seq`**. Y eso **es exactamente la vista dividida** que la
> firma existe para hacer oponible.
>
> **El modo de fallo del esquema y el fraude perseguido son el mismo
> evento.**

La observacion es de B15 de `CONFIANZA_RESIDUAL.md`; lo que esa propuesta no
conecto es que **B10.1 depende de ella** (§103.3).

### 106.5 Lo que esta decision NO resuelve

- **No implementa nada.** Es un criterio de eleccion, y la firma sigue sin
  existir (entrada 53).
- ⚠️ **El estado de XMSS hay que persistirlo.** Un nodo que reinicie y pierda
  el indice **puede reusar uno**, y entonces filtra su clave. Con `seq`
  derivada del log persistido el riesgo baja, pero **no se ha comprobado que
  el log garantice esa monotonía a traves de un reinicio**. Entrada 18 ronda
  cerca.
- **No dice el tamaño del arbol XMSS**, que fija cuantas epocas se pueden
  firmar antes de agotar las claves. Es un parametro con consecuencia: **un
  arbol agotado deja al operador sin poder publicar cabezas.**

## 107. El 2-de-N de `circuit_mint`, comprobado — y dos mediciones malas

Especificando `circuit_mint` aparecio una pregunta que el codigo no
responde: **¿que impide que un custodio firme dos veces?**

§80 lo enuncia como riesgo —*«un 2-de-N en el que un custodio pudiera contar
dos veces seria un 1-de-N disfrazado»*— y `mint.rs` de la capa lo comprueba
con `index_a >= index_b`. Pero **la capa no es el circuito**, y §73 registra
que pasa cuando una propiedad la impone solo la capa.

⚠️ **No habia ningun test que lo probara.**

### 107.1 La defensa existe, y esta donde no se esperaba

No hay ninguna restriccion que compare los indices. La defensa es la
**descomposicion binaria por segmentos**, la misma que impone el tope de
emision:

```rust
expected[7] = current[COL_IDX_B] - current[COL_IDX_A] - E::ONE;
```

Con `IDX_A == IDX_B` eso vale `-1`, que en Goldilocks es `p-1` y **no cabe en
el segmento**. La prueba no verifica.

✅ **MEDIDO**: `one_custodian_cannot_sign_twice` →
`Err("verificacion fallo: InconsistentOodConstraintEvaluations")`.

> **La prueba se genero, se verifico, y el verificador la rechazo.** La
> defensa esta en una restriccion, **no en el constructor de la traza** — que
> es exactamente la distincion que §73 costo aprender.

⚠️ **Y depende de tres cosas separadas**: `C_ACC` acumula los bits del camino,
`C_ACC_FINAL` ata el indice acumulado al declarado, y el segmento impide la
diferencia negativa. **Juntas dan la propiedad; ninguna la da sola.** Es la
tercera «garantia por consecuencia» del proyecto, junto a las dos de
`circuit_burn` (§105.2).

### 107.2 ⚠️ La primera medicion era VACUA por construccion

El test se escribio con `prover.prove(trace).is_err()`.

> **En release el probador no valida las restricciones** (§77.1): genera la
> prueba igual, y es el verificador quien la rechaza. **`prove()` no puede
> fallar en release**, asi que ese test habria «detectado» un agujero en
> **cualquier** circuito, correcto o no.

Y lo habria registrado como el tercer fallo de solidez del dia.

### 107.3 ⚠️ Y la segunda medicion tambien: acerte por el motivo equivocado

Se sospecho del test **porque tardo 0,05 s**, razonando que «una prueba STARK
son cientos de milisegundos».

**Esa referencia era falsa.** Los 52 tests de `circuit_mint` corren en 1,36 s;
los cientos de milisegundos eran de `circuit_settlement`, que es mucho mayor.

> **Un tiempo sin referencia no es una medida.** La sospecha era correcta y su
> razon no: lo que delataba el test era **su logica** —`prove` no falla en
> release—, no su duracion.

⚠️ **Y si hubiera tardado 300 ms, no habria sospechado nada.** El error se
caza por leer lo que el test comprueba, no por cronometrarlo.

### 107.4 Lo que esto dice de B12.1

La especificacion de `circuit_mint` **no encontro un fallo**, pero encontro
**una defensa que nadie habia comprobado** y que ahora tiene test.

> El valor del ejercicio no esta en documentar lo que hay: esta en que
> **obliga a preguntar «¿que impide X?»** por cada cosa que el enunciado
> promete. Y esa pregunta, aqui, no tenia respuesta escrita en ninguna parte.

## 108. ⚠️ §99.4 descarto mal el salt en el estado

§99 cerro la entrada 50 diciendo que **no hay solucion conocida**: derivar el
salt de la clave es imposible porque `open_account`, `mint`, `freeze` y
`recover` escriben la hoja **sin conocer el secreto**. Eso sigue siendo
cierto.

Pero §99.4 descarto ademas, **sin medir**, la familia «salt en el estado»,
con este razonamiento: *«la capa lo escribe, luego lo ve»*.

**Ese descarte esta mal, por dos razones distintas.**

### 108.1 Aplico el criterio equivocado

*«La capa lo ve»* es cierto **y es irrelevante**:

| | ve el salt | ¿importa? |
|---|---|---|
| El operador | si | **no** — ya ve los saldos, declarado en el README |
| **El vecino de arbol** | **no** | ⚠️ **es la entrada 50** |

> **La entrada 50 es sobre TERCEROS, no sobre el operador.** Un salt que el
> operador conozca y el vecino no **cierra la 50 sin pretender cerrar lo que
> nunca se prometio.**

§99.4 exigio «ocultante frente a todos» a un problema medido «frente a
terceros».

### 108.2 Y no distinguio DERIVAR de CONSERVAR

La capa **no necesita computar el salt**. `mint` compone la hoja asi:

```rust
native_leaf(updated.public_id, BaseElement::new(updated.balance), updated.nonce)
```

desde `AccountRecord`, **sin entender ninguno de sus campos**. Un salt seria
uno mas: lo guarda y lo pasa, igual que pasa `public_id`.

> **Conservar no es derivar.** §99.3 conto bien quien escribe hojas sin la
> clave, y de ahi salto a que el salt era incomputable — cuando basta con que
> sea **transportable**.

### 108.3 Lo verificado, pieza a pieza

| | |
|---|---|
| `AccountRecord` | tres campos; **48 bytes fijos** en disco → 80 con salt, **migracion necesaria** |
| `AccountView` —lo que lee **cualquiera** por `account_view`— | **el salt NO tiene por que entrar ahi** |
| `ClientState` —del titular— | ahi si; y ⚠️ **`state_of` vive en `tests_support.rs`**: **no existe como API de la capa**, asi que el canal **hay que diseñarlo** — y puede exigir autorizacion desde el primer dia |
| `SendMaterials` | lleva la vista **del propio emisor**, y del receptor **solo el identificador** |
| El camino Merkle | **hashes de hoja**, no sus componentes — con salt dejan de ser diccionariables |

### 108.4 Lo que NO resuelve

- **De donde sale el salt al abrir cuenta**, y si el titular puede
  recuperarlo si lo pierde. **§93.4 sigue en pie**: el cliente no custodia
  estado hoy, y esto se lo pediria.
- **El coste en circuito**: `native_leaf` gana un argumento y toca los cinco
  de gasto. Es **clase entrada 15**, con el coste ya medido en §86 —ensanchar
  **abarata**—.
- ⚠️ **No es una solucion: es una familia que vuelve a estar viva.**

### 108.5 ⚠️ Y la duda que va con esta correccion

Es la **novena autocorreccion del dia**, y todas siguen el mismo patron: una
conclusion escrita con rigor resulta estar mal **al intentar usarla**.

§93.5 dio un obstaculo falso. §99 lo corrigio y dio otro. §108 corrige el
descarte de §99. **Las tres se escribieron con la misma seguridad.**

> Admite dos lecturas y no se cual es la buena: **que el metodo funciona**
> —cada conclusion se somete a uso y las malas caen— **o que se concluye
> demasiado rapido**, y la proxima revision encontrara algo tambien.
>
> Se registra la duda junto a la correccion. Quien lea §108 debe ver que
> corrige a §99, que corrigio a §93.

## 109. La 49 y la 50 NO se resuelven con la misma pieza

Con §108 devolviendo vida al «salt en el estado», se propuso usar **el salt
como credencial de lectura**: seria una llave que —a diferencia de la
identidad publica que §95.4 descarto— **no es la direccion de pago** y el
titular puede derivar de `sk`.

**La propuesta era mala.** Se descarta antes de escribir codigo, y se
registra por que.

### 109.1 Un secreto que viaja en cada operacion no es una credencial

El cliente **necesita el salt cada vez** que reconstruye su hoja para probar.
Va y viene por el canal operador↔cliente constantemente.

> Una credencial deberia viajar lo minimo. **Un cegado tiene que viajar cada
> vez.** Son requisitos opuestos sobre el mismo valor.

### 109.2 Y no se podria rotar

Si el salt se compromete hay que rotarlo. Pero:

1. Rotarlo **cambia la hoja**.
2. Cambiar la hoja **es una transicion de estado**.
3. Que exigiria **autorizacion**… con el salt comprometido.

> **Un secreto que ciega el estado y autoriza el acceso al estado no se puede
> rotar sin usarlo.** Es circular.

### 109.3 ⚠️ Y la razon de fondo: la 49 no es un problema criptografico

`account_view`, `balance_of`, `public_id_of` y `nonce_of` —**cuatro puertas,
no una**— devuelven datos sin comprobar quien llama.

> **La solucion de un problema de control de acceso es control de acceso.**
> No reutilizar material criptografico porque este a mano.

⚠️ Y habria creado **la cuarta garantia por consecuencia** del proyecto:
lectura y cegado atados al mismo secreto, dependiendo el uno del otro **sin
que nada avise si uno cambia**. Las otras tres estan en §105.2 y §107.1, y
las tres son la forma de §72.

### 109.4 ⚠️ Y el salt no arregla la 49 de todos modos

`balance_of(index)` devuelve **el numero**. Un salt oculta la hoja frente a
quien la ve **hasheada** —el vecino con su camino Merkle— **no frente a quien
pregunta**.

| entrada | de quien protege |
|---|---|
| **50** | del vecino que **recibe un camino** |
| **49** | del que **simplemente pregunta** |

**Son ataques distintos**, y arreglar la 50 sin la 49 dejaria hojas cegadas
**y una API que da el saldo en claro**: la defensa criptografica seria
decorativa.

### 109.5 ⚠️ CORRECCION de §95.4: la 49 no es barata

§95.4 dijo que la mitigacion de la 49 «se cierra facil» y que solo era
discutible **como declararla**.

**Es falso.** El proyecto **no tiene ningun mecanismo de autenticacion de
titular**:

- No hay firma (§103.1).
- `open_account` **no registra nada** que sirva para probar identidad
  despues: solo guarda `public_id`, `balance` y `nonce`.
- Y la unica prueba de titularidad que existe es **un STARK de ~600 ms**
  (§93.1), inaceptable para una lectura.

> **La 49 depende de la 53** —la firma XMSS— **o de un mecanismo
> equivalente.** No es una tarde.

## 110. XMSS convierte una perdida de durabilidad en perdida de secreto

Al ir a implementar la firma (entrada 53) aparecio una dependencia que
ninguna de las cuatro secciones anteriores tenia.

### 110.1 ⚠️ La 53 depende de la 19, y hace media hora se dijo lo contrario

El orden de prioridad escrito hace treinta minutos decia que la 53 era **«lo
unico de arriba que puede empezarse sin decidir nada mas»**.

**Es falso.** XMSS necesita que su indice se persista **antes de firmar**, y
eso es un requisito sobre la persistencia que el proyecto **no cumple hoy**.

**Undecima correccion del dia**, y la primera de algo dicho en la misma hora.

### 110.2 El argumento de la entrada 19 deja de valer

`persistence.rs` documenta por que no hay log de escritura anticipada, con
una tabla precisa de que se pierde en cada momento de fallo. Y cierra:

> *«Lo que se pierde en el caso intermedio es **durabilidad** —la operacion
> puede haberse perdido— pero no **integridad**. **Perder una operacion es
> recuperable: se vuelve a enviar.**»*

**Ese razonamiento es correcto para el ledger y falso para XMSS:**

| | perder una operacion |
|---|---|
| Hoy | recuperable — se vuelve a enviar |
| **Con XMSS** | ⚠️ **la clave de firma se filtra** al reusar el indice |

> **XMSS convierte una perdida de durabilidad en una perdida de secreto.**

La decision de no tener WAL esta **bien razonada para el sistema actual** y
deja de estarlo en cuanto se firme con indice derivado de `seq`. No es que
la entrada 19 estuviera mal: es que **XMSS cambia su premisa**.

### 110.3 ⚠️ Y el reuso de indice es AMBIGUO desde fuera

§103.3 y §106.4 celebran que el modo de fallo de XMSS **sea** el fraude
perseguido: reusar indice significa dos cabezas con el mismo `seq`, que es la
vista dividida.

**Eso solo vale si el reuso es fraude.** Un reinicio honesto tras un fallo
entre firma y persistencia produce **el mismo evento**:

> **«Te pille mintiendo» y «acabas de perder tu clave» son indistinguibles
> para un observador externo.**

⚠️ Y con un agravante: **la cabeza ya esta publicada**. Un testigo la tiene.
El operador no puede deshacer la firma ni explicar la coincidencia sin pedir
que le crean — que es exactamente lo que las cabezas atestiguadas existen
para no tener que hacer.

**Eso no esta en §106 y es lo primero que un auditor preguntaria.**

### 110.4 El patron: cuatro capas, ninguna en la clasificacion original

| pieza | clasificada como | resulto depender de |
|---|---|---|
| B10.1 | «aditiva, sin dependencias» | una **firma** que no existe (§103.1) |
| La firma | «componente» | el **esquema**, que es decision de tesis (§106.2) |
| XMSS | «formato» | **persistir el indice** (§106.5) |
| Persistir el indice | — | un **WAL** que no hay (§110.1) |

> `CONFIANZA_RESIDUAL.md` clasifica esfuerzos **sistematicamente por
> debajo**, y van **cuatro piezas seguidas** donde ocurre. No es que la
> propuesta sea mala: es que **una propuesta escrita sin implementar no ve
> las capas de abajo**, y eso es informacion sobre como leerla — no solo
> sobre esta pieza.

### 110.5 Lo que sigue en pie

**XMSS sigue siendo la eleccion correcta** (§106.3): no añade familia de
supuestos, y ese criterio no cambia.

Lo que cambia es el **orden**: antes de firmar hay que resolver que el indice
no retroceda nunca. Y hay dos vias, ninguna medida:

- **WAL** —entrada 19—: persistir el indice antes de firmar.
- **Indice independiente y monotono**, persistido con `fsync` propio antes de
  cada firma, desacoplado de `seq`. ⚠️ Pierde la propiedad de §103.3 —que el
  reuso sea la vista dividida— pero **§110.3 acaba de mostrar que esa
  propiedad era ambigua de todos modos**.

## 111. La 53 NO depende de la 19 — y la 19 afirma algo que no existe

### 111.1 ⚠️ §110.1 fue precipitada

Hace quince minutos §110.1 concluyo que **la 53 depende de la 19**, porque
XMSS necesita persistir su indice antes de firmar y el proyecto no tiene WAL.

**Son piezas distintas:**

| | que necesita | quien lo da |
|---|---|---|
| **Entrada 19** | durabilidad del **ledger** | `sled` ya la da; un WAL propio **añadiria superficie**, y `persistence.rs` lo argumenta bien |
| **XMSS** | que **su indice** no retroceda nunca | **un contador propio con `fsync`**, aislado del ledger |

Lo segundo es **una pieza pequeña**: un valor, persistido antes de cada
firma. No toca el ledger, no reimplementa nada de `sled`, y **no tiene la
superficie de fallo que `persistence.rs` teme de un WAL**.

⚠️ **Y §110.3 quito el motivo de acoplarlos.** La propiedad elegante de
§103.3 —que reusar indice **sea** la vista dividida— resulto ambigua, porque
un reinicio honesto da el mismo evento. **Sin esa propiedad, atar el indice a
`seq` no compra nada**, y un contador independiente es mas simple.

**Duodecima correccion del dia**, y **tercera dentro de la misma hora en que
se escribio lo corregido**.

### 111.2 ⚠️ Y la entrada 19 afirma una salvaguarda que NO existe

La entrada dice:

> *«Un fallo entre operaciones **detiene el arranque pidiendo intervencion
> manual**: correcto, pero no automatico.»*

**Medido: cero coincidencias** de `intervencion`, `manual` o
`StoreError::Malformed` en toda la capa. **Nada detiene el arranque.**

Y `persistence.rs` documenta lo contrario:

> *«En los tres casos el ledger queda **coherente**… se pierde durabilidad,
> no integridad.»*

⚠️ **La parte comprobable de la entrada es cierta** —no hay WAL, y ella misma
da el `grep` que lo demuestra—. **La descripcion de la consecuencia es
falsa.**

### 111.3 Es la quinta de la familia §95.2, y de la peor clase

| | la forma |
|---|---|
| §76, §84.2, §95, §98.4 | una **garantia** cuya **condicion** nadie verifico |
| **§111.2** | una **salvaguarda que no existe** |

> Las cuatro anteriores prometian de mas sobre algo real. **Esta describe un
> mecanismo inventado** — y un lector que confie en ella creera que el
> sistema se protege de un fallo del que **no se protege**.

⚠️ Y lo mas incomodo: la entrada **se acompaña de su propio `grep`
comprobable**, que es la disciplina de la casa. **Comprobar la mitad
verificable de una afirmacion y dar la otra mitad por buena es exactamente lo
que §59.2 castiga**, cometido en el documento que lleva la cuenta.

## 112. Evaluacion XMSS ejecutada: que, con que prueba, y que se descarto

Se evaluo `xmss` 0.1.0-pre.0 —**RustCrypto/signatures**, publicado el
25-06-2026— con las cinco pruebas de la entrada 53, mas agotamiento y medidas
por conjunto. Detalle en `doc/xmss-evaluacion.md`.

⚠️ **El campo cambio despues de escribir el protocolo**: ese crate es de la
misma organizacion que `sha2` y `chacha20poly1305`, **ya aceptadas por
politica** en el `Cargo.toml`. Pre-release y **sin auditoria independiente**,
declarado por el propio crate.

| criterio | resultado |
|---|---|
| 1 · solo hash | ✅ `cargo tree` = **0 curvas ni reticulos**. ⚠️ `chacha20` via `rand` para el RNG de keygen — cifrado de flujo, **sin supuesto nuevo** |
| 2 · indice | ⚠️ **persiste, pero la API no lo expone**: solo derivable del formato de referencia —SK 136 B, OID `0x00000001`, indice en `[4..8]` BE—. Fragil: en MT mide ⌈h/8⌉ |
| 3 · reuso | ❌ **firma** — dos firmas validas al mismo indice. Y `Clone` en `SigningKey` lo reproduce **sin tocar disco** |
| 3b · agotar | ✅ `KeyExhausted` en la firma **2^h exacta**, sin dar la vuelta |
| 4 · altura | ✅ h=10 y MT 20/2. Las alturas son **menu discreto**: 10/16/20 y 20/40/60 |
| 5 · revisada | ⚠️ suite 10/10, **funcional**; KAT sin confirmar. A favor: **conformidad de formato byte a byte** —firmas de 18.469 y 27.688 B, las cifras del RFC— |

**Descartes**: `purecrypto`, **por declaracion de su propio autor** —*«do not
use it for anything real yet»*—; el XMSS de QRL queda fuera del marco por ser
C++ via FFI.

⚠️ **Y `purecrypto` enseño algo sobre la prueba 1**: un **monolito derrota el
`cargo tree`**, porque las curvas viven dentro del crate y el arbol no las ve.

### 112.1 El criterio 5 pedia algo que no existe

Pedia «vectores de RFC 8391» y **el RFC no publica vectores**. Los canonicos
son los de `xmss-reference` y los ACVP de SP 800-208.

> Es **§95.2 en nuestro propio protocolo**: una condicion enunciada que nadie
> habia verificado, escrita el mismo dia que la seccion que castiga esa forma.

## 113. Ninguna candidata cumple el criterio 3 — y ninguna puede

El criterio exigia que restaurar un estado previo y firmar devolviera error.

**Era insatisfacible junto al criterio 2.** Un estado serializado a bytes del
llamante —lo que el criterio 2 **exige**— y restaurado es **indistinguible de
uno legitimo**: detectar el retroceso requiere informacion **externa al
blob**.

> SP 800-208 asume lo mismo: por eso **exige modulos hardware** para firmas
> con estado.

⚠️ **Decimotercera correccion del dia, y la primera de algo redactado como
criterio para terceros.** El protocolo se escribio veinte minutos antes de
evaluarlo con el.

**Reformulacion**: **3a** firma por `&mut` que avanza antes de devolver, sin
API firmar-con-indice —✅ aqui— y sin `Clone` silencioso —✘, registrado—;
**3b** fallo duro al agotar —✅—. **La proteccion contra retroceso es siempre
del proyecto.**

### 113.1 El guardian deja de ser contingencia: es el mecanismo

Contador propio, `fsync`, **firmar-despues-de-persistir**, con las dos
cautelas intactas —`fsync` puede mentir; el orden invierte el flujo natural—
mas dos requisitos medidos:

- **Test de layout**: el guardian lee el indice en el offset del formato de
  referencia y debe llevar un test que lo compruebe, para que **un cambio de
  serializacion falle en CI y no en produccion**.
- **Reconciliacion tras reinicio**: si el contador propio supera al indice del
  SK, el indice huerfano **se quema o se registra la divergencia**; nunca se
  retrocede.

⚠️ **Se declara, como el protocolo exigia: la seguridad del esquema pasa a
depender de codigo propio, NO auditado.**

## 114. La firma cuesta O(d·2^(h/d)): la tercera victima

**Medido**: firma ≈ keygen —645 vs 634 ms en h=10, constante en 1024 firmas—
y **SK de 136 B sin estado de recorrido** ⇒ la implementacion **reconstruye
el arbol en cada firma**. Modelo ~0,62 ms/hoja, validado con error < 1 % en
dos predicciones.

| conjunto | firma | verif | tamaño |
|---|---|---|---|
| **MT 40/8** | **160,5 ms** | 2,7 ms | 18.469 B |
| MT 60/12 | 241,2 ms | 3,8 ms | 27.688 B |
| XMSS 20 | ≈ 650 s *(derivado)* | — | 2.820 B |

**Es propiedad de la implementacion, no del esquema** —BDS la eliminaria— y
depende de un **pre-release**: puede cambiar upstream. Va en el issue
(`doc/issue-rustcrypto.md`).

### 114.1 ⚠️ ENMIENDA: la tabla NO queda «sin incognitas»

`doc/xmss-evaluacion.md` §5 concluye que deja la decision «sin incognitas».
**No es exacto**, y la diferencia importa: **toda la evaluacion asume una
firma por segundo**, y esa premisa no se cuestiono.

| cadencia | conjunto | almacenamiento | CPU |
|---|---|---|---|
| **1/s** | 40/8 | **0,58 TB/año** | 16 % de un nucleo |
| **1/min** | 40/8 | **9,7 GB/año** | 0,27 % |
| **1/min** | **40/4** | **5,2 GB/año** | 4,2 % |

> A una firma por minuto el almacenamiento **cae 60 veces** y **`40/4` vuelve
> a ser viable**. La recomendacion de `40/8` es correcta **bajo la premisa**,
> y la premisa es lo que no se discutio.

⚠️ **Y no es un parametro: es cuanto tarda una vista dividida en ser
oponible.** Firmar cada 60 epocas la detecta igual —dos cabezas
contradictorias siguen siendo dos cabezas— con granularidad de un minuto.
**Certificate Transparency no firma cada segundo.**

**Decision con victimas** (`PRINCIPIOS.md`): cadencia fina cuesta 0,58 TB/año;
cadencia gruesa da al operador **una ventana de un minuto** para servir
historias distintas. **Queda abierta.**

### 114.2 Procedencia de estas medidas

Las cifras vienen etiquetadas con maquina y modo —`--release`, x86-64,
01-08-2026— que es la disciplina de la casa. ⚠️ **No se han reproducido de
forma independiente**, y los pendientes 1 y 2 de la evaluacion —grep de los
diez tests, fijar el commit del tag— son lo que las cerraria.

Y los 0,62 ms/hoja son **de esa maquina**: si el nodo objetivo es ARM (B9),
repetir antes de fijar nada.

## 115. Cadencia de firma: 1/min + a demanda. Premisa MEDIDA

**Decision**: las cabezas se firman **una vez por minuto**, y ademas **a
demanda** — toda peticion externa se responde firmada, con **cache por
`seq`**: maxima una firma nueva por epoca, idempotente. Conjunto
`XMSSMT-SHA2_40/8_256`.

⚠️ **El peor caso adversarial esta acotado**: un testigo pidiendo cada
segundo degenera **exactamente** al escenario 1/s ya cuantificado —16 % de un
nucleo, agotamiento a ~35.000 años—. **Techo conocido, no regimen.**

### 115.1 La premisa decisiva estaba razonada y ahora esta medida

La decision descansa en que **una cabeza firmada en `n` ata tambien las
epocas anteriores** via `chain_digest`: la ventana de 60 s es **latencia de
oponibilidad, no impunidad**.

⚠️ **Eso se enuncio como razonamiento, no como medida** —y se señalo antes de
decidir, que es la diferencia con las cinco veces que §95.2 registra—.

**Medido**: `log.rs`, modulo `t1_chain_retroactivo`, **3/3 en verde**:

| test | que demuestra |
|---|---|
| `t1_divergencia_localizada` | divergir en `k` contamina todo `j ≥ k`, y `first_divergence` localiza `k` **exacto** |
| `t1_cabeza_ata_la_historia` | un testigo que recomputa desde los campos crudos reproduce `head()`, y **alterar la epoca `k` falsifica la cabeza de `n`** |
| `t1_verify_chain_caza_sustitucion` | la sustitucion se detecta **incluso en un log restaurado de copia** |

> El segundo es, literalmente, *«la firma en `n` cubre la mentira en `k`»*.

### 115.2 Lo que la decision cierra

| | |
|---|---|
| Politica de retencion de firmas | **muere sin decidirse**: 9,7 GB/año se retienen integros durante decadas |
| CPU de firma | del **16 % al 0,27 %** de un nucleo |
| Estado BDS del issue upstream | de **necesidad a mejora** |

### 115.3 Condicion de reversion, y un juicio declarado

⚠️ **Sigue viva**: si el sistema promete algun dia oponibilidad **sub-minuto
sin peticion** —firmeza en caliente estilo RTGS—, se paga el escenario 1/s
con sus **0,58 TB/año** y su retencion. **Cambio de mision, via
`PRINCIPIOS.md`.**

⚠️ **Juicio declarado, no medido**: que este sistema responde a escalas de
supervision y conciliacion —donde 60 s no compran nada a un operador
deshonesto— es **una premisa sobre el uso**, no un hecho del codigo. **Si el
uso real cambia, cae la premisa y la decision con ella.**

## 116. ⚠️ `digest_of_proof` rellena con ceros sin codificar la longitud

Hallazgo colateral de §115: el resumen que ata cada entrada del log a su
prueba **rellena el ultimo bloque con ceros y no codifica la longitud del
mensaje**. **Dos pruebas que difieran solo en ceros finales colisionan.**

### 116.1 Por que NO se degrada a nota al pie

Su propia doc lo llama «atar, no hash de proposito general», y el asiento
original lo degradaba porque **no habilita fabricar pruebas validas
alternativas**.

**Eso es cierto hoy y depende de algo externo al resumen**: de que el
verificador rechace la prueba alterada.

> ⚠️ **Es la CUARTA garantia por consecuencia del proyecto** —§105.2 (dos en
> `circuit_burn`), §107.1 (`circuit_mint`)—: una propiedad que se sostiene
> **porque otra pieza la sostiene**, sin que nada avise si esa pieza cambia.

Y con un agravante que las otras tres no tienen:

> **El `chain_digest` que §115.1 acaba de validar incluye
> `digest_of_proof`.** La cadena que ata toda la historia —y que ahora
> sostiene la decision de cadencia— es tan fuerte como ese resumen.

**«Atar a una prueba concreta» es falso al byte**, y T1 mide el
encadenamiento, **no la resistencia a colision del resumen**.

**Arreglo**: bloque final con la longitud del mensaje. Entrada 58.

## 117. El salt de hoja se deriva de la clave. Cierra §108.4

**Decision**: `salt_hoja = native_merge([LEAF_SALT_DOMAIN], clave_ancha)`,
como `derive_leaf_salt_wide` / `derive_leaf_salt` en `circuit_settlement.rs`.
La estrecha rellena `[sk,0,0,0]` y hereda §90: **dos vias de apertura, una
cuenta, UN salt** — comprobado, no prometido.

### 117.1 Por que derivarlo y no las otras dos vias

| origen | por que no |
|---|---|
| Aleatorio custodiado por el titular | ⚠️ **§93.4 lo mata**: «el cliente no custodia estado hoy, y esto se lo pediria» |
| Suministrado por el operador | muere contra la mision de **operador ciego** |
| **Derivado de la clave** | ✅ satisface §93.4, la migracion futura y la promesa vigente **a la vez** |

> **El salt no puede perderse por separado — solo con la clave, y entonces ya
> estaba todo perdido.** Cero secretos de perdida fatal nuevos.

### 117.2 Sometido a uso: T2a, 3/3

⚠️ **`t2a_dominio` es la apuesta real**, y su motivo merece leerse: el
nullifier es `merge(merge([NULLIFIER_DOMAIN], clave), [nonce])`, asi que una
colision de dominios **haria del salt el estado interno del nullifier** — una
**llave maestra de trazabilidad de todos los gastos de la cuenta**.

Los otros dos: `t2a_anchura_coherente` hereda §90 en las dos vias, y
`t2a_determinista_no_trivial` comprueba que la posicion importa
—`[sk,0,0,0] ≠ [0,0,0,sk]`—.

### 117.3 ⚠️ CORRECCION al coste 3 del asiento: no es divergencia de estilo

El asiento anota como coste que `LEAF_SALT_DOMAIN` use **8 bytes** donde la
casa usa 4 —`"SPKY"`, `"NULL"`, `"CUST"`, `"GOVE"`—.

**Medido: hay 5 dominios distintos, y la separacion es ESTRUCTURAL.**

> `LEAF_SALT` mide 8 bytes y los otros cuatro miden 4. **No pueden colisionar
> por longitud**, no solo por valor.

Eso es **mas fuerte que lo que el test comprueba**: `t2a_dominio` verifica
que los valores elegidos no chocan; la longitud garantiza que **no podrian**.

**No es un coste: es una propiedad**, y conviene que quede escrita como tal
para que nadie la «arregle» por consistencia de estilo.

### 117.4 Lo que sigue abierto, con nombre

- **T2b, test de aceptacion de la 50** —no de esta entrada—: abrir cuenta,
  **destruir todo el estado del cliente salvo la clave**, re-derivar, gastar.
  ⚠️ **Si T2b no puede escribirse, esta decision cae.**
- **El asterisco de §93.4**: la clave **ya llega** a la capa en
  `open_account(_wide)` —no se *almacena*, pero se *ve* al abrir—. La v1
  puede derivar el salt en el nodo **sin que fluya un bit nuevo**. Existia
  antes de esta decision; esta entrada **lo documenta, no lo crea**.
- Renombrar en doc `salt_hoja` / `salt_aviso`: el de `pending.rs` es **otro
  animal** —por pago, aleatorio, del emisor, viaja en el aviso—.

## 118. ⚠️ Dos documentos publicos dicen «sin solucion conocida»

`SECURITY.md` §3.2 y `doc/preprints/ZK-SSL-residual-trust.md` §4.7 afirman
que la fuga de la entrada 50 **no tiene solucion conocida**, y que derivar el
salt de la clave **esta descartado**.

**§108 y §117 lo desmienten.** Mientras no se corrijan, el proyecto tiene
**dos documentos publicos contradiciendo lo decidido** — uno de ellos
depositado con DOI.

⚠️ **Y la correccion no es «ya esta resuelto»**: es que **el cegado es frente
a TERCEROS y solo frente a terceros**, con la v1 derivando el salt en el
nodo. Decirlo de menos seria falso; decirlo de mas, tambien.

**No se corrige aqui**: los preprints estan **suspendidos** (entrada 28), y
`SECURITY.md` espera a que T2b confirme la decision.

## 119. La reversion es un segundo cobro, jamas una tercera voluntad

**Decision** (pieza 4 de §88.4): quien devuelve es **el emisor, con su clave,
tras un plazo**, por la misma maquinaria de claim, hacia una identidad de
retorno comprometida en el envio.

### 119.1 Lo que mata las alternativas es una sola frase

> Operador, gobernanza, custodios caso a caso **y el barrido automatico**
> —*«el mismo poder con eufemismo»*— serian **la primera transicion del
> sistema donde el dinero se mueve SIN LA CLAVE DE UN TITULAR**.

Eso rompe la fila «gastar de una cuenta ajena» de la tabla de garantias.
**Muertas por identidad del proyecto, no por conveniencia.**

⚠️ Y **«nadie, nunca» sobrevive** — como el subcaso **Δ=∞ pago a pago**, no
como politica. Es mas honesto: el statu quo se conserva como **eleccion del
emisor**, no como imposibilidad.

### 119.2 El mecanismo, y su invariante

El compromiso pendiente v2 ata `(receiver_id, salt, amount)` **mas**
`refund_id` —elegido por el emisor— **y los terminos temporales**.

> ⚠️ **Invariante**: los terminos van **en el compromiso**, no solo en el
> aviso. Inmutables desde el envio y **a la vista del receptor**, que ve al
> recibir cuando puede vencer lo suyo.

Antes del vencimiento **solo el receptor cobra**; despues, **tambien** el
retorno. **El receptor nunca pierde el derecho hasta que el emisor ejecuta**:
no hay fondos que caduquen al vacio ni barredor. La invariante de suministro
no se toca.

### 119.3 ⚠️ El reloj: `seq` NO es un reloj

**Autocorreccion registrada por el propio asiento.** Se afirmo que «`seq` es
inflable porque `open_account` no exige autorizacion»; el codigo muestra
`max_accounts` acotandolo.

Y esa correccion tapaba **la de fondo**:

> **`seq` cuenta operaciones, no tiempo.** En un sistema quieto, un mes son
> **cero entradas** — y §88.5 pedia «meses, no bloques».

**El plazo se cuenta en cabezas de epoca firmadas** —el latido 1/min de
§115—: un reloj anclado a calendario **cuya aceleracion por el operador deja
evidencia firmada y oponible** en manos de cualquier testigo. `seq` crudo no
la dejaba.

La pieza 2 de §88.4 queda precisada: **no es «meter seq en la traza», es atar
el input temporal a una cabeza verificable.**

### 119.4 §88.6 resuelto, con su coste

El circuito de reversion lleva `frozen_root` y comprueba **receptor ∉
congelados** — espejo de lo que `circuit_claim` hace con el cobrador (§29).
Congelar preserva el patrimonio **entrante** entero: «enviarle y revertir»
no lo vacia.

⚠️ **Coste con nombre**: la congelacion pasa a **retener fondos de terceros**
—los emisores—, lo que añade presion a la politica de caducidad de
congelaciones que el README ya lista como falta.

### 119.5 Poderes nuevos del operador

1. ⚠️ **Tras Δ hay carrera** emisor/receptor **y el operador ordena**.
   Combinando orden y censura puede forzar el desenlace entre los dos
   legitimos. **No puede robar** —el dinero solo va a receptor o retorno—
   pero es poder nuevo.
2. El reloj de cabezas firmadas convierte su aceleracion en **evidencia
   oponible**.

⚠️ **3. Y uno compuesto que el asiento no separa**: el operador es **a la vez
quien ordena la carrera y quien acelera el reloj**. Puede censurar al
receptor durante Δ y luego favorecer al emisor.

> Cada pieza deja evidencia por separado —la censura no esta cerrada, la
> aceleracion es oponible— pero **combinadas producen un resultado que
> ninguna de las dos evidencias explica sola**. No cambia la decision; **es
> una fila propia**, no una mencion de pasada.

### 119.6 Sin retroactividad — estructural, no disciplinaria

**Los pendientes v1 son irreversibles para siempre**, incluidos los perdidos
de §30.

> Cambiarles los terminos a posteriori seria **exactamente la deshonestidad
> que el proyecto existe para impedir.**

✅ **Comprobado en arbol**: `t3a_sin_retroactividad_por_construccion` —ningun
`refund_id` con ningun reloj reconstruye un compromiso v1—.

Hacia delante, v2 **cierra las tres fuentes de la entrada 12**: el receptor
lento cobra hasta que el emisor ejecuta; el identificador inexistente de §30
pasa de perdida segura a **recuperable si Δ es finito**; el limbo de §29
queda **retenido-durante y reversible-despues**.

### 119.7 Sometida a uso: T3a, 3/3 sobre el struct real

Exclusion mutua sobre el arbol real, solo el retorno designado revierte —con
el pendiente **integro** tras el intento ajeno—, y la no-retroactividad.

⚠️ **El compromiso v2 del test es prototipo**: el formato real llega con las
piezas 1-3, y **T3b es su aceptacion** —(a) antes del vencimiento rechaza;
(b) despues verifica y consume; (c) receptor congelado ⇒ rechazada; (d) un v1
rechazado **tambien en circuito**; (e) el input temporal atado a una cabeza
firmada—.

**Si (a) no puede escribirse** porque el reloj no entra como input publico,
la politica queda **condicionada, no caida**.

### 119.8 La linea para `PRINCIPIOS.md`

> **Δ_min = 13 meses.** Ningun pago puede llevar un plazo de reversion menor.

**Precedente**: el plazo de *recall* de SEPA. ⚠️ **Con la analogia imperfecta
declarada**: el recall exige consentimiento del receptor y esto es
**unilateral por abandono**, lo que argumenta un suelo **largo, nunca corto**.

> **Un Δ≈0 seria un pago que nunca fue promesa.**

Δ por encima del suelo lo elige el emisor, con **Δ=∞ como statu quo**.

## 120. ⚠️ El codigo cita dos documentos que NO EXISTEN

### 120.1 Medido

`CONFIANZA_RESIDUAL.md` y `ESCALADO.md` **no estan en el arbol ni en el
historial** —`git log --all` no devuelve nada—. Y se citan:

| dónde | citas |
|---|---|
| **Codigo publicado** | **6** — `lib.rs:422`, `log.rs:405/423/446`, `metrics.rs:73/194` |
| `AUDITORIA.md` | **12** |
| `BACKLOG.md` | **6** |

**Veinticuatro referencias a documentos que nunca se commitearon**, seis de
ellas **en el codigo**, por nombre y seccion: `B10.1`, `§8.1`, `§2.2`,
«dimensiona el shard sobre 4 ms».

### 120.2 El propio proyecto tiene la frase que lo condena

`ZK-SSL-residual-trust.md`, en voz del autor:

> *«Un numero que no describe nada ejecutable es una **dependencia residual
> de la confianza del lector en el autor**.»*

⚠️ **Una cita a un fichero inexistente es exactamente eso** — y esta en el
codigo publicado, no en notas internas. Un lector que quiera comprobar
`log.rs:423` **no puede**.

### 120.3 ⚠️ Cuatro de las seis citas del codigo son de HOY

`lib.rs:422` y `log.rs:405/423/446` se escribieron esta sesion, al construir
`EpochHead` (§104). **Se cito un documento que estaba pegado en el chat como
si fuera del repositorio, sin comprobar que lo estuviera.**

> Es §101 exacto —inventariar los ficheros equivocados por no comprobar cual
> era el objeto— **cometido otra vez seis horas despues, en el mismo dia y
> por la misma mano.**

### 120.4 Sexta de la familia §95.2, y de clase nueva

| | la forma |
|---|---|
| §76, §84.2, §95, §98.4 | una **garantia** cuya condicion nadie verifico |
| §111.2 | una **salvaguarda que no existe** |
| **§120** | una **referencia que el lector no puede seguir** |

Las anteriores prometian de mas sobre algo real o describian algo ausente.
**Esta ofrece una comprobacion y la hace imposible** — que es peor en un
proyecto cuyo metodo es la trazabilidad.

### 120.5 La decision, ABIERTA y declarada

| | |
|---|---|
| **A · commitear los dos documentos** | cierra las 24 citas. ⚠️ Mete en el arbol **texto nunca revisado y con cifras desfasadas** —el borrador dice «cabeza ~200 B» y la real son **~18,5 KB** (§114)— |
| **B · repuntar cada cita** | mas fiel al estado real. ⚠️ **Algunas no tienen destino**: `§8.1` y `§2.2` no estan en ningun fichero, ni en el preprint derivado |

**Recomendacion registrada, no ejecutada**: **A con cabecera de estado** —
commitearlos **marcados como propuestas de sesion no revisadas**, con la
lista de lo que otras secciones ya corrigen.

> B convierte 24 citas verificables en 24 parafrasis y **pierde la
> trazabilidad que este proyecto usa como metodo**. A la conserva, **al
> precio de que el arbol gane dos documentos de autoridad menor** — y eso
> hay que decirlo en su cabecera, o se repite el problema de los preprints.

**No se ejecuta aqui**: es decision del autor.

## 121. El acuse compromete su propio N, bajo techo

**Decision** (B10.3): **N va comprometido en cada acuse**, inmutable y a la
vista de quien lo recibe.

### 121.1 Lo que resuelve, y la simetria con §119

⚠️ El problema declarado era que **un N corto convierte congestion legitima
en falsa evidencia**. Con N comprometido por acuse:

> Bajo congestion, el operador honesto **no viola un N corto: emite acuses
> con N mayor y lo declara**. La congestion deja de fabricar falsa evidencia
> y pasa a ser **degradacion visible y firmada** del servicio.
>
> Y **el censor que infla N tambien se delata**: un N obeso es señal que
> cualquier testigo lee.

| | quien elige | que lo acota |
|---|---|---|
| §119 · reversion | el **emisor** | un **suelo** que protege al receptor |
| §121 · acuse | el **operador** | un **techo** que protege al usuario |

**Una sola linea sistemica en cada caso.** Aqui: `N_max`.

### 121.2 ⚠️ La trampa del borrador: NADA de firma-por-acuse

B10.3 decia que el operador **firma** `(hash_de_la_prueba,
epoca_de_recepcion)` al aceptar.

> **Tal cual, a miles de operaciones por segundo, revienta la aritmetica de
> la entrada 53**: los 2^40 indices de XMSS durarian **semanas**.

**Correccion de diseño, no de politica**: los acuses son **hojas bajo una
raiz de recepcion comprometida en la cabeza**. El acuse individual = camino
Merkle + la cabeza firmada del latido.

> **Hereda la firma. Cero indices extra.** Y llega en ≤1 latido —≤1 min—,
> despreciable frente a `N_max`.

⚠️ **Es la SEGUNDA extension pendiente de `EpochHead`** —tras
`verifier_hash` (§104.3)— **antes de que exista un solo testigo**. La cabeza
aun no esta desplegada, asi que no hay migracion; pero conviene contarlas.

### 121.3 El detector que NO necesita N

El acuse lleva un **contador de recepcion monotono**. Entonces:

> *«Una operacion con contador posterior a la mia entro y la mia no»* es
> evidencia de **reordenacion inmune a la congestion** — porque **la
> congestion retrasa a todos; solo la censura adelanta**.

**Detector inmediato**, y N queda como cota de **abandono**, no de lentitud
—el mismo corte que §88.5—. Coste: un `u64`.

### 121.4 El reloj ya estaba decidido dos veces

N se cuenta en **cabezas firmadas**, no en epocas crudas (§115, §119.3):
`epoca_de_recepcion` = `seq` de la ultima cabeza firmada al aceptar.
**Estirar el latido para esquivar N es en si evidencia oponible.**

Y la regla **a demanda** de §115 regala la mitad del mecanismo: acuse +
secuencia de cabezas sin la inclusion **es un registro oponible del retraso
sea cual sea N**.

### 121.5 ⚠️ Cruces con otras secciones

- **§116**: el acuse ata `hash_de_la_prueba`. Si usa `digest_of_proof` tal
  cual, **hereda la colision por ceros finales**. Exige el digest con
  longitud codificada, o un hash real. **§116 se cierra antes que esto.**
- **Entrada 53**: los costes del borrador —«cabeza ~200 B»— **predatan la
  firma**: la cabeza real son **~18,5 KB** (§114).
- **Nomenclatura**: reservar «acuse». El proyecto ya tiene dos `receipt`
  —`SendReceipt`, `GovernanceReceipt`, que son **otro animal**: paquetes de
  prueba del cliente—. Un tercero homonimo repetiria lo de los salts.

### 121.6 La linea para `PRINCIPIOS.md`, propuesta

> **N_max = 1.440 cabezas firmadas** —24 h al latido de 1/min—: ningun acuse
> puede prometer inclusion mas lenta.

**Precedente**: el MMD de Certificate Transparency, 24 h. **Aritmetica
medida**: a 70 ms por `apply` sin lotes, 24 h absorben **mas de 1,2 M de
operaciones** de backlog — **a ese horizonte, «era congestion» muere como
defensa**.

⚠️ **Caveat declarado**: en CT la consecuencia era ecosistemica —desconfiar
del log—; **aqui el sistema produce el par condenatorio, no adjudica**. La
consecuencia es del supervisor o del contrato.

### 121.7 Limite heredado, sin maquillar

> **El acuse solo encarece censurar lo acusado.** Negarse a emitir acuses es
> el mundo pre-B10.3: **detectable por el cliente afectado** —«no me dan
> acuse» es razon para no operar— **pero no evidencia portable**.

Limite del borrador, que N no cambia y esta entrada no maquilla.

### 121.8 Sometimiento: documental, y por que

⚠️ **Primera de las cuatro decisiones del dia cuyo sometimiento no fue
mecanico**: el acuse **no existe en codigo**, asi que no habia nada que
ejecutar. Se sometio el **texto**: recuperado del registro de sesiones,
colacionado contra el codigo publicado —los `receipt` existentes son otro
animal— y con dos piezas del borrador **corregidas antes de heredarlas**
(§121.2 y §121.5).

**T4 es su aceptacion mecanica**: (a) inclusion en ≤N ⇒ no hay par
condenatorio construible; (b) no-inclusion tras N ⇒ acuse + N cabezas
verifica como evidencia portable; (c) contador `i` incluido con `i−1` fuera
⇒ evidencia **inmediata**; (d) un acuse no es confundible con los `receipt`
existentes; (e) ata la prueba con digest con longitud.

**Si (a)-(b) no pueden escribirse** porque la raiz de recepcion no entra en
la cabeza, el hueco es el formato de B10.1 — **trabajo, no obstaculo**, y la
politica queda **condicionada, no caida**.

## 122. Los dos fantasmas, committeados. Y el encaje, aplicado con VISION §5

**A endurecida ejecutada** (§120.5): `doc/ESCALADO.md` y
`doc/CONFIANZA_RESIDUAL.md`, **cuerpo verbatim**, con cabecera que **no es
descargo sino mapa de correcciones**.

✅ **`tools/verificar_citas.py`: 30 nombres citados, 0 fantasmas.** Las 24
citas vuelven a ser verificables.

### 122.1 El encaje se decidio con un instrumento que YA EXISTIA

`VISION.md` §5 es literalmente un criterio de aceptacion de cambios futuros.
**No hubo que inventar el test de encaje: hubo que aplicarlo.**

⚠️ Que este proyecto tuviera ya escrito como decidir esto, y que nadie lo
usara en las horas que se pasaron discutiendo ambos documentos, **es el
mismo patron que §120**: un instrumento existente que no se busca.

### 122.2 `ESCALADO.md` — encaja como DOS cosas que el documento mezcla

| | veredicto |
|---|---|
| **C2, C3, C6** | ✅ sirven a la mision declarada **a cualquier escala**. C3 —entradas publicas *hiding*— es la **prioridad 1 de VISION** y la pieza que `CONFIANZA_RESIDUAL` llama B11 |
| **El dimensionamiento 5×10⁹** | ⚠️ es el estudio de **una mision que el proyecto no ha declarado** |

**Y la confesion esta en su propio §6**: para la mision declarada —200
instituciones— *«3,3 tx/s: el codigo actual la ejecuta hoy, sin cambios»*.

`PRINCIPIOS.md` P4 —«ampliar el alcance no es un bien en si»— **no prohibe el
estudio: prohibe adoptarlo sin decision consciente**, y esa decision es de
mision y pasa por VISION §5, no por el backlog.

### 122.3 Dos casi-choques que la colacion desactivo

**Uno · homonimia.** VISION §5.1 **rechazo** «agrupar pruebas por lotes»
—falla el criterio 3: exigiria claves de gasto— y **C4 comparte palabra**.

> **No comparte naturaleza**: C4 **verifica cada prueba individualmente** y
> lotea solo arbol y disco. Sin agregacion, sin claves.

⚠️ Hay que decirlo o **el catalogo de rechazos parecera violado**. Es la
tercera homonimia del proyecto, tras los salts (§117) y los `receipt`
(§121.5).

Y §89.2 le regala a C4 **un argumento mejor que el suyo**: verificar es el
**3,2 %** del `apply`, asi que el lote ataca exactamente **el 96,8 % que
domina**.

**Dos · C1.** Declara el nullifier «bloqueante para cualquier escala», y
VISION §4-p7 ya lo matizo: **solo afecta a `transfer()`**; `send`/`claim` no
usan nullifiers, y **retirar la via antigua lo cierra hoy sin tocar
circuitos**.

### 122.4 ⚠️ DEUDA CON NOMBRE: la contencion del anclaje

`ESCALADO.md` §2.2 identifica **la contencion del anclaje de raiz** como su
limite n.º 1, con **1,6 TPS efectivos**.

**Medido: cero coincidencias** de esa cifra en `AUDITORIA.md` y en
`README.md`. **No esta registrada en ninguna parte.**

> ⚠️ **Debe entrar al registro AUNQUE EL ESCALADO NO SE ADOPTE JAMAS.** Es
> el criterio 6 de VISION aplicado: un limite medido del sistema actual no
> depende de si se adopta la propuesta que lo encontro.

Entrada 63.

### 122.5 `CONFIANZA_RESIDUAL.md` — el mapa corre AL REVES

**Encaja de lleno, y no como ampliacion**: VISION §1 pide «el poder del
intermediario acotado, visible y minimo», y **este documento es ese verso
operacionalizado**. B10 corre sobre el nodo unico **hoy**.

⚠️ **Su mapa no corrige cifras hacia abajo: anota que la realidad lo
adelanto.**

| el documento propone | el estado real |
|---|---|
| B10.1 cabeza firmada | **construida, firmada y medida** — 40/8, 18,5 KB, 160 ms (§104, §114) |
| B10.2 comparacion entre testigos | **primitiva con test**: `first_divergence` + T1 (§104.2, §115.1) |
| B10.3 acuse | **decidido** en §121, con dos correcciones al borrador |
| B8 / §87 | **resuelto** en §119 |
| «la errata del salt», precondicion de B18.2 | **resuelta en §117 — y MEJORA B18**: el reclamo de migracion exige **solo la clave** |

**Coherencia fina que cierra**: §5.3 prohibe el rollback automatico y **§119
no lo viola** — la reversion es un claim con clave de titular, **segunda
voluntad, jamas mutacion sin prueba**.

### 122.6 ⚠️ El desfase que el guardian NO caza

El codigo cita `§8.1` y `§2.2` de `CONFIANZA_RESIDUAL.md` **con la
numeracion v1**. El documento commiteado es la **v2**, donde esas secciones
son **§10.1 y §10.2**.

> **El guardian valida que el FICHERO existe; no que la SECCION exista
> dentro.** Es la misma clase de cita imposible de seguir, **un nivel mas
> abajo**.

### 122.7 Y el patron que produjo los fantasmas sigue activo

`Downloads` contiene `asiento-115`, `-116`, `-117`, `-119` y dos
`auditoria-asientos` *(sin extension a proposito: ver 122.8)* — **todos
entregables de sesion cuyo contenido ya
vive en este fichero**, y uno de ellos genero el tercer fantasma.

> **La fuente del problema no fue un descuido puntual: es como se trabaja.**
> Se generan ficheros de trabajo que se integran **por contenido**, y cuyo
> **nombre** puede acabar citado.

El guardian lo caza en CI. **La practica sigue siendo la misma.**

### 122.8 ⚠️ El guardian no distingue CITAR de NOMBRAR

Al registrar §122.7 —*«el patron que produjo los fantasmas sigue activo»*— se
**nombro** el fichero de ejemplo, y el guardian lo conto como **cita**.
**Tercera aparicion del mismo nombre, y la tercera por la misma via.**

> **La seccion que describe el patron lo cometio al describirlo**, en el
> mismo commit que lo registra.

**Limite del guardian, declarado**: su ambito dice «solo `.rs` y `.md` son
fuentes de citas», y eso acota **donde** mira. **No acota que una mencion no
es una referencia.** Aqui se resuelve quitando la extension; una version
futura podria ignorar los nombres dentro de bloques marcados.

⚠️ **Y su otro limite sigue vivo** (entrada 64): valida que el **fichero**
exista, **no que la seccion citada este dentro**.

## 123. La contencion del anclaje: mecanismo firme, numero en rango

Cierra la entrada 63 y la deuda de §122.4. Tests en `client.rs`, modulo
`t5_contencion_anclaje`.

### 123.1 El mecanismo, comprobado en su forma dura (T5a)

Dos titulares que **no comparten nada** —cuentas distintas, hojas distintas,
cero estado comun— generan contra la misma raiz. Aplicar la primera **mata a
la segunda**: `Err(LayerError::StaleState)`.

> **El anclaje global serializa operaciones independientes.** Es **propiedad
> del sistema actual**, no de la propuesta que la encontro.

⚠️ **Esta parte es cualitativa y firme.** No depende de ninguna constante.

### 123.2 ⚠️ El numero varia entre ejecuciones, y por eso va en rango

| | primera | segunda |
|---|---|---|
| `t_gen` | 375 ms | **314 ms** |
| `t_apply` | 90 ms | **62 ms** |
| **TPS efectivo** | **1,53** | **1,87** |

**22 % de diferencia entre dos corridas de la misma maquina.**

> ⚠️ Publicar «~1,5 TPS» repetiria **exactamente lo de `ESCALADO.md`**: un
> TPS derivado de constantes que no han pasado el protocolo de §89.1.

### 123.3 El modelo anterior acerto por las razones equivocadas

`ESCALADO.md` §2.2 daba **1,6 TPS** ≈ 1/620 ms.

> **Un modelo con las dos constantes equivocadas que cayo dentro del rango
> real por compensacion de errores.**

⚠️ **Y eso es indistinguible de acertar hasta que se mide.** Es §99.5 en
forma numerica: un dato correcto sobre el objeto equivocado.

**Correccion de una etiqueta propia**: §122.4 la llamo «limite medido».
**Era prematuro.** Desde §123 lo es.

### 123.4 §22 arbitrado por medida directa

§22 daba `apply` en **177 ms** —28,5 % de un 620 que tampoco era—. Medido:
**62-90 ms**. **§22 cae**; §89.2 tenia la historia correcta con el numero
corto.

### 123.5 ⚠️ Deuda: `t_gen` canonico, y el README ya la lleva marcada

**375 y 314 ms discrepan del 620 canonico** de `README.md`, `PRINCIPIOS.md`
y `doc/ESCALADO.md`. **No se corrige el canon**: n=15 no es el protocolo de
§89.1.

⚠️ **Pero la cifra publicada deja de ser firme**, y el README lo dice ya.
**Entrada 65.**

### 123.6 ⚠️ Y aparecio un SEPTIMO sitio de la unidad

`README.md` decia **120,4 MB** para una cifra binaria. El inventario de
§84.5 hablaba de **seis** sitios y la correccion del 01-08 cubrio los tres
documentos del lote — **el README no estaba en el lote**.

> **Segunda vez que ese inventario se queda corto**: §100.3 registro que
> `QUESTIONS.md` se habia quedado fuera del primer parche. **Ahora el
> README.**

### 123.7 El metodo, para el registro

El bloque se ejecuto dos veces; en la segunda **la guarda md5 aborto el
pegado** y evito un modulo duplicado — la enfermedad de §89.3, donde un
`#[test]` duplicado sesgo su propia medida un 19 %.

> **El ABORTA era el sistema funcionando.**

## 124. `digest_of_proof` inyectivo — y la segunda familia que §116 no vio

Cierra la entrada 58 y desbloquea §121.5.

### 124.1 ⚠️ El arreglo registrado en §116 NO habria bastado

§116 registro una familia de colision —ceros finales, distinta longitud— y
escribio su arreglo: «bloque final con la longitud».

**Al implementarlo aparecio la segunda familia**, que ese arreglo no cubre:

> La reduccion `% p` hacia que un bloque de 8 bytes con valor **exactamente
> `p`** colisionara con uno de ceros **de la misma longitud**. **La longitud
> sola no bastaba.**

⚠️ **Es §95.2 otra vez** —una correccion registrada cuya condicion nadie
verifico— **pero cazada antes de committearse**. Es la primera del dia que
se detiene en esa fase.

### 124.2 El arreglo, y que abarata

**Codificacion inyectiva de raiz**: limbs de **32 bits** —cada elemento
< 2³² < p, **sin reduccion al campo**— empaquetados de a 4 por merge, mas
bloque final de longitud.

| | antes | ahora |
|---|---|---|
| Bytes por permutacion | 8 | **16** |
| Merges para 62 KB | 7.750 | **3.876** — la mitad |
| Coste medido | — | **28,06 ms** (n=20) |

**Relleno de ceros + longitud explicita = codificacion libre de prefijos.**
La resistencia a colision queda **donde debe**: en `native_merge` (Rescue).

⚠️ **Contexto del coste**: 28 ms frente a los **62-90 ms** del `apply`
(§123.4) — **en torno a un tercio**. No despreciable, no dominante.

Y el comentario del codigo deja de decir «no pretende atar al byte»:
**ahora atar a una prueba concreta es verdad al byte**.

### 124.3 El corte, en la unica ventana en que es gratis

`chain_digest` incluye `proof_digest`, y las cabezas firmadas comprometen
`chain_digest`: **esto es cambio de formato del log**.

**Radio de explosion comprobado antes de cortar:**

- `digest_of_proof` **solo lo llama `append`** y los tests.
- `LogEntry` **no deriva serde**.
- `persistence.rs` **no contiene la palabra `chain`**.

> **Nada almacenado en ninguna parte codifica el digest viejo.** Hoy el corte
> es gratis; **con acuses emitidos o logs persistidos habria sido
> migracion**.

⚠️ **Sin compatibilidad hacia atras a proposito**: un modo compatible
**conservaria la debilidad**.

**Y es la primera vez en esta sesion que el orden de las decisiones sale a
favor.** §121 decidio el acuse **y no lo implemento**; si lo hubiera hecho,
esto seria una migracion.

### 124.4 Sometido a uso — T6, 4/4

| test | que mata |
|---|---|
| `t6_ceros_finales` | la familia de §116, bordes de bloque incluidos |
| `t6_aliasing_del_campo` | ⚠️ **el par `p ≡ 0` de misma longitud** — la segunda familia |
| `t6_a_nivel_de_chain` | la amenaza real: dos pruebas casi-iguales, **dos cabezas** |
| `t6_coste_sobre_prueba_realista` | 28,06 ms, n=20 |

✅ **Y la suite entera como prueba del radio**: los **213 verdes siguen
verdes** —T1 y T3a incluidos, asi que **la propiedad retroactiva de §115 se
sostiene sobre el digest nuevo**—. Con T6: **217**.

Los 3 rojos son los de `tests_privacidad`, **rojos por construccion e
independientes del digest**.

### 124.5 Lo que desbloquea, y la nota honesta

**§121.5 satisfecho**: el acuse ya tiene con que atar `hash_de_la_prueba` —
**nace sobre el digest inyectivo, sin heredar deuda**.

⚠️ Y lo que el asiento dice de si mismo: los 3 rojos de superficie **quedan
mas visibles que nunca** — son la **prioridad 1 de VISION** esperando turno.

## 125. Coherencia por construccion o por test, nunca por costumbre

Cierra las entradas 59, 60 y 64 con **un solo principio**, que no estaba
escrito y deberia:

> **La coherencia entre piezas que deben coincidir o es por CONSTRUCCION
> —una sola definicion— o es por TEST —guardian—. Por costumbre, nunca.**

Las tres deudas eran **la misma enfermedad con tres cepas**.

### 125.1 La 59 se desmintio POR LECTURA

El miedo de §117 era «dos implementaciones de Rescue que deben coincidir y
nada lo comprueba».

**La lectura lo desmintio de la mejor manera**: los dos son **el mismo
wrapper caracter a caracter** sobre la `Rp64_256` de winter-crypto.

> **No habia dos Rescue. Habia una fotocopia.**

Y eso hace el cierre **trivial en vez de arriesgado**:
`rescue_hash::native_merge` **delega** en `merkle::native_merge` — una
definicion, **cero call-sites tocados**, y el modo de fallo —editar una copia
y no la otra— **deja de existir**.

⚠️ El miedo registrado era desproporcionado, y solo se supo **leyendo los dos
cuerpos**.

### 125.2 La 60, la grave, cerrada por construccion

Dos copias de `NULLIFIER_DOMAIN` podian divergir y dar **nulificadores
distintos para el mismo gasto** segun el circuito — **doble gasto entre
vias**, la garantia mas cara del sistema rota en silencio.

**Cierre**: una definicion canonica por dominio —`nullifier.rs` para
NULLIFIER, `circuit_threshold` para CUSTODIAN— y `pub use` en los demas. Las
rutas viejas resuelven; **el valor ya no puede divergir**.

⚠️ **Retirada honesta incluida**: los dos `assert_eq!` que T2a puso el dia
del censo **se volvieron tautologicos** al deduplicar.

> **Un test que no discrimina es una garantia falsa.** Se retiran, con la
> nota en su lugar — y **retirarlos es mas dificil que dejarlos**.

**Frontera declarada**: las replicas de `halo2`/`plonk`/`zk-core` siguen por
copia **fuera** del arbol de dependencias de `zk-ssl`.

### 125.3 La 64: una cita viva, y la clase cerrada

El censo encontro **una sola** cita de codigo con seccion desfasada:
`log.rs` citaba §8.1 con la numeracion v1 — corregida a **§10.1**.

Y la clase queda con guardian: `verificar_citas.py` **v2** comprueba que toda
cita de la forma «*fichero* `.md` §N» desde `.rs` apunte a **una seccion
que exista**.

✅ **Primer veredicto**: `0 fantasmas · 0 secciones muertas · exit=0`.

### 125.4 ⚠️ Pero el ambito del guardian v2 deja fuera el 90 % de las citas

Su docstring declara **«solo `.rs`»**, con el argumento de que *«las
cabeceras-mapa narran numeraciones viejas a proposito y no son rot»*.

**Correcto para las cabeceras. Y deja sin cubrir el resto:**

| | citas de seccion | ¿cubiertas? |
|---|---|---|
| Codigo `.rs` | ~10 | ✅ |
| `AUDITORIA.md`, `BACKLOG.md` | **cientos** | ❌ |
| Cabeceras-mapa | ~30, **viejas a proposito** | ❌ correcto |

> **Se excluye por FICHERO en vez de por BLOQUE**, y eso deja fuera **la
> mayor superficie de citas del proyecto** para proteger tres cabeceras.

Las citas de `AUDITORIA.md` son **el tejido del registro**: `§93.4`,
`§105.3`, `§121.5`. Si una apunta a una seccion que se renumero, el registro
se rompe por dentro **y nadie lo sabe**.

**Entrada 66.**

### 125.4-bis ⚠️ Y el guardian volvio a cazar una MENCION

El ejemplo de §125.3 usaba un nombre generico entre comillas. **El guardian
lo conto como cita** — **cuarta vez** que confunde nombrar con citar, tras
`auditoria-asientos` en §122.8.

> **La seccion que documenta el limite del guardian lo desencadeno al
> documentarlo.** Es §122.8 literal, repetido dos secciones despues.

Confirma la entrada 66: el arreglo no es evitar los ejemplos, es **excluir
por bloque marcado**, no por extension de fichero.

### 125.5 Dos errores de metodo, documentados

**El del asiento**: el bloque del parche llevaba una etiqueta perdida que
**rompio el comando final**, y el guardian quedo sin correr en esa pasada.

> Se detecto **porque el rito exige VER el resumen, no suponerlo**. Misma
> familia que §89.3: **el instrumento tambien se audita**.

⚠️ **Y uno mio**: una lectura de `cargo test` **durante la recompilacion**
dio `277 passed; 1 failed`, y se sospecho una regresion en `native_merge`
—la funcion que compone todos los hashes—. **Tres pasadas siguientes: 278
limpio.**

> **Un resultado de test leido mientras `cargo` recompila puede mentir.** Es
> §107.3 otra vez: **una cifra sin su condicion de medida no es una medida.**

## 126. T2b partido en dos, y la clausula de caida resuelta A FAVOR

§117 llevaba escrita su propia condena: *«si T2b no puede escribirse, esta
decision cae»*.

### 126.1 El hallazgo que forzo partir el test

**La hoja se computa DENTRO de la traza**: `circuit_send.rs` fuerza la
permutacion Rescue de `native_leaf` como restricciones periodicas del AIR
—`ROW_LEAF_LINK`, `C_LEAF_CAP/DIG`—.

> Meter el salt es **cambio de AIR en los cinco circuitos de gasto**, no de
> capa. **T2b tal como se especifico exige B13/B14 ya hechos**, y escribirlo
> hoy seria **un test que no compila disfrazado de verde**.

### 126.2 La distincion que salva la coherencia

⚠️ **La clausula no dice** «§117 cae si el test no puede escribirse hoy» —eso
seria **rehen de un calendario de ingenieria**—. **Dice** «cae si la
propiedad que el test afirma es **irrealizable**».

> Separarlas evita **las dos** patologias: **fingir el verde** y **tumbar una
> decision solida por falta de andamiaje**.

### 126.3 T2b-nativo: la propiedad, sin el circuito — 3/3

`circuit_settlement.rs`, `mod t2b_recuperacion_nativa`, sobre `native_leaf`
y `native_merge` reales. Introduce `native_leaf_salted` como definicion
**nueva** —la vieja y sus 60+ call-sites **intactos**: eso es B13/B14—.

| test | que demuestra |
|---|---|
| `t2b_solo_la_clave_reconstruye_la_hoja` | ✅ **decide la clausula**: un titular que pierde todo salvo la clave rederiva identidad y salt (§117) y **reconstruye su hoja identica** |
| `t2b_diccionario_sin_salt_no_acierta` | el ataque **medido** de la 50 —10,84 s— **no acierta ninguno** con salt |
| `t2b_hoja_vieja_sigue_expuesta` | la 50 se cierra **hacia delante**, no sobre hojas ya escritas |

⚠️ **El segundo lleva su control**, y esa es la parte que importa: comprueba
que el barrido **no acierta sin el salt** *y* que **si reproduce la hoja con
el salt correcto**.

> Sin ese control, **un cegado que rompiera tambien al legitimo daria el
> mismo verde**. Es §66.2 aplicado: el positivo primero y solo.

### 126.4 ⚠️ Veredicto: §117 SE SOSTIENE. Y lo que eso NO significa

La 50 pasa de **«familia viva»** —su etiqueta desde §108— a **propiedad
demostrada con despliegue pendiente**.

⚠️ **No esta cerrada.** Y hay una consecuencia que `t2b_hoja_vieja_sigue_
expuesta` convierte de aspiracion en **comportamiento verificado**:

> **Las cuentas abiertas antes del despliegue siguen siendo barribles en
> 10,84 s, y eso no lo arregla ningun salt futuro.**

Es la forma de §98.2 —la rotacion existe— **con una diferencia**: alli se
rotaba para ganar **entropia**; aqui **hay que rotar para ganar la
privacidad**. Una cuenta vieja que nunca rote **queda expuesta para siempre**.

### 126.5 Lo que queda, tasado

**T2b-circuito**, condicionado a B13/B14: el `apply` extremo a extremo con el
salt ya presente en las cinco trazas. **Especificado, NO escrito** — se
documenta como pendiente honesto, **no se finge**.

**Coste del despliegue**: clase entrada 15, con precedente medido —§82
ensancho la clave de 1 a 4 elementos en traza, y §86 midio el coste
**negativo**: ensanchar abarata—. `native_leaf_salted` es **un merge Rescue
mas por hoja**.

### 126.6 La guarda, otra vez

El md5 de partida del asiento era el del **arbol de lectura**, no el estado
**post-§125**. **La guarda lo detecto.**

> Es la tercera vez en dos horas que el ABORTA evita construir sobre un
> estado que ya no existia.

## 127. La 49 esperaba a la 53 para ver que la 53 no la resuelve

### 127.1 La 53 cierra, con su juicio ratificado COMO JUICIO

Todo lo tecnico estaba hecho (§112-115). Lo unico vivo era **un juicio
declarado**, y se ratifica como lo que es:

> Que **60 s de latencia de oponibilidad no compran nada a un operador
> deshonesto** es una **premisa sobre el USO** del sistema —liquidacion
> institucional supervisada, no pagos minoristas en caliente—, **no un hecho
> del codigo**. Puesta por quien decide, **revisable si la mision cambia**.

⚠️ **Reversion viva**: prometer oponibilidad **sub-minuto sin peticion**
cuesta el escenario 1/s —**0,58 TB/año**—. Cambio de mision, via VISION §5.

### 127.2 ⚠️ Mi error de tipo: la dependencia era al reves

§109.5 escribio *«la 49 depende de la 53»*, esperando que la firma resolviera
la autenticacion.

**Con la firma ya elegida se ve el error:**

| | quien firma | que |
|---|---|---|
| **XMSS (53)** | **el operador** | cabezas de epoca — **una identidad**, la clave del nodo |
| **La 49 necesita** | **cada titular** | prueba de titularidad de **SU** cuenta |

> **La 53 tenia que resolverse antes para ver que NO resuelve la 49.**

La dependencia se cierra y la 49 pasa a necesitar **pieza propia**.

### 127.3 Decision: clave de vista, y por que esta

**`derive_view_key(sk)`** — credencial de lectura derivada de la clave de
gasto, patron de §117 aplicado a lectura. El titular la presenta; la capa la
compara contra un `view_id` guardado al abrir.

| opcion | veredicto |
|---|---|
| derivada + verificada **en circuito** | ❌ la prueba de ~600 ms **que la propia 49 rechaza** |
| derivada + verificada **nativamente** | ✅ **elegida**: un merge, y **NO viaja en cada operacion** —a diferencia del salt que §109 descarto por eso— |
| **secreto nuevo independiente** | ❌ rotable de verdad, pero es el **modo de perdida fatal nuevo** que §117 rechazo y la custodia que §93.4 prohibe |

⚠️ **Se guarda `view_id` = hash de la clave de vista, NO la clave**: el
operador queda con material para **COMPARAR**, no para **LEER**.

### 127.4 T7, 3/3 — y el test que hace segura la separacion

| test | que demuestra |
|---|---|
| `t7_solo_el_titular...` | el titular reproduce su `view_id`; un tercero sin la clave, no |
| `t7_la_vista_no_es_credencial_de_GASTO` | ⚠️ presentar la vista **no deriva la clave de gasto ni la identidad** |
| `t7_dominio` | dominio distinto de los cinco vivos, **`LEAF_SALT` incluido** |

> **Sin el segundo, autenticar la lectura seria entregar el gasto.**
>
> Y el tercero cubre una interaccion fina que no estaba en ningun sitio: **si
> los dominios coincidieran, presentar la vista revelaria el salt de hoja de
> §117** — y eso **reabriria la 50 por la puerta que se acaba de cerrar**.

Tercera vez que el proyecto usa separacion de dominios, y **la primera que se
comprueba entre dos piezas nuevas**.

### 127.5 ⚠️ La limitacion, y su consecuencia

La clave de vista **esta acoplada a la clave de gasto**, asi que **rotarla
exige rotar la de gasto**. El asiento lo declara; la consecuencia conviene
subrayarla:

> Si la clave de vista se compromete, **el titular no puede recuperar la
> privacidad de lectura sin cambiar de cuenta**.

Es la forma de §110.2 con XMSS: **un secreto derivado hereda el destino del
que lo genera.** Se elige el acoplamiento **conscientemente**, sobre la
alternativa de un secreto nuevo de perdida fatal.

### 127.6 Mitad B: la predecibilidad del indice, que NINGUNA credencial arregla

`account_indices_are_not_predictable` lo mide desde §93.3: **las altas dan
indices consecutivos**, asi que quien controla el momento de su alta **ELIGE
a su vecino de arbol, y con dos altas lo rodea**.

> Convierte la fuga de la 50 de **oportunista** en **DIRIGIDA**.

Es **ortogonal a la autenticacion** —aleatorizar la posicion de alta, o
derivarla del `public_id`, es su propio cambio— y llevaba **desde §93 sin
entrada propia**.

### 127.7 Alcance honesto: DECISION + NUCLEO, no cierre

T7 demuestra que el mecanismo es **realizable**. **La 49 NO esta cerrada.**

El cierre toca `AccountRecord` —ganar `view_id`—, `open_with_id`, **las
cuatro firmas publicas** y ~100 call-sites de test. **Eso es despliegue.**

⚠️ Se declara para **no vender como resuelto lo que es decidido-y-fundado**
—criterio 5 de VISION—. Y sigue viva la observacion de §124: **los tres
rojos de `tests_privacidad` son exactamente lo que la 49-A pondra en verde.**

## 128. Ocho items, tres fisicas — y tres que NO se cierran

⚠️ **La decision de esta tanda es no tratarla como un lote.** Los ocho items
tienen fisica distinta, y meterlos en un cierre comun seria el error de bulto
que este proyecto evita.

### 128.1 Grupo A — T3b y T4: partidos, como §126

Ni T3b ni T4 deciden nada: **verifican que las politicas de §119 y §121 son
implementables**. Al leerlos contra el arbol, **ambos exigen andamiaje
inexistente** —T3b el circuito de reversion con `refund_id`; T4 el acuse como
hoja bajo raiz de recepcion—. **Escribirlos enteros hoy seria fingir verde.**

| | resultado |
|---|---|
| **T3b-nativo 2/2** | la reversion tras plazo, con el **reloj en cabezas firmadas** (§119.3), y **solo el `refund_id` legitimo** la reconstruye. Mas la no-retroactividad estructural |
| **T4-nativo 2/2** | el acuse liga `(hash_prueba, epoca, N)` **con digest inyectivo** (§124), y el **contador monotono distingue reordenacion** (§121.3) |

✅ **Las clausulas de §119 y §121 se resuelven a favor**: las politicas son
implementables; **el circuito es ingenieria, no riesgo de existencia**.

**T3b-circuito y T4-circuito**: especificados, condicionados y **NO escritos**.

### 128.2 Grupo B — la entrada 54, registrada donde faltaba

⚠️ **El poder mayor del operador no figuraba en NINGUNA de las dos
enumeraciones del proyecto**: ni en `SECURITY.md` §2 —que lista ver,
censurar, ordenar y custodiar— ni en la tabla §4.1 de `residual-trust`.

> **Cambiar el verificador redefine que es una transicion valida.** Ordenar,
> censurar y observar actuan **dentro** de las reglas; sustituir el
> verificador **cambia las reglas**.

✅ **Aplicado en `SECURITY.md` §2**, con su cierre nombrado y su precondicion:
el sistema **no tiene nocion de «reglas vigentes»**, asi que el campo de la
cabeza **no puede rellenarse hoy** — y un campo vacio seria peor que su
ausencia (§104.3).

⚠️ **El parrafo para el preprint queda REDACTADO Y SIN APLICAR**: los
preprints estan **suspendidos** (entrada 28), y editarlos reabriria por la
puerta de atras lo que se suspendio. Ademas **ya estan depositados**: no
llegaria a ningun lector hasta una quinta revision. **Se guarda en la entrada
54.**

### 128.3 Grupo C — tres que NO se cierran, y por que declararlo es la decision

⚠️ **Estas tres no se resuelven en esta tanda**, y decirlo es **mas honesto
que un cierre fingido** —criterio 5 de VISION—.

| | por que no, y siguiente paso |
|---|---|
| **52** | **Toca la palabra del titulo**: puede gastar sin permiso de nadie y **no puede protegerse sin permiso de dos**. ⚠️ Antes de proponer rotacion autoautorizada hay que **entender por que la recuperacion exige custodios** — probablemente la misma razon. **Diseño nuevo, sesion propia.** |
| **32** | Inventario medido: **138 llamadas + 145 usos de `open_and_fund`** —que llama a `.mint()` por dentro: **media suite depende de la via antigua sin nombrarla**—. ⚠️ **El precio de la delegada —tres pruebas donde la antigua hace una— NO esta medido.** Trabajo mecanico grande. |
| **47** | Falta mapear **B1-B9 contra el backlog** —su B8 es la 12, ya resuelta en §119— y aplicar las cuatro correcciones de §89.4. **Reconciliacion, no decision.** |

### 128.4 Metodo: dos patrones propios, corregidos

1. **Etiqueta perdida cerrando bloques de parche** (§125, §127): **dos veces
   rompio el comando final**. Regla: los heredoc terminan en su delimitador,
   **sin texto detras**.
2. **`tail -N` sobre salida de cargo**: **tres veces oculto el resumen real**
   —cargo emite el bloque de la lib **y** el de doc-tests, y el ultimo es el
   vacio—. Regla: **`grep "test result"`, nunca `tail`**.

> Son hermanos: **descuidos de lectura del instrumento**, cazados porque el
> rito exige **VER, no suponer**. La misma red que caza los errores del
> proyecto (§89.3), funcionando sobre los propios.

## 129. 49-A: lo que es de verdad, medido antes de tocar codigo

### 129.1 ⚠️ Dos premisas mias, desmentidas por el arbol

**Una · «pone en verde los tres testigos rojos».** **Falso, y el propio
codigo lo dice:**

| testigo | ¿lo cierra la clave de vista? |
|---|---|
| `reading_a_balance_requires_authority` | ✅ **si** |
| `account_indices_are_not_predictable` | ❌ es **49-B** —posicion de alta—, **ortogonal a toda credencial** |
| `a_neighbour_leaf_does_not_reveal_its_balance` | ❌ es la **50**, y su propio cuerpo lo declara: *«esta fuga sobrevive a cerrar `account_view`»* |

**Indicador corregido: UN rojo verde, no tres.** Fingir tres seria el
autoengaño que la casa evita.

**Dos · «exigir credencial en las cuatro puertas».** **Rompería el sistema,
no solo la suite.** Medido: **97 usos de `balance_of` en once ficheros**, y
**no solo en tests** —13 en `iso.rs`, 7 en `two_phase.rs`: camino de
produccion—.

> **`balance_of` es infraestructura interna**, y el operador **ya ve los
> saldos**: exponerselos a el **no filtra nada**.

El contrato que el rojo pide no es «toda lectura exige credencial», es **«la
vista destinada al titular exige que sea el titular»**. Se **AÑADE** una
puerta autenticada; **no se lisian los accessors**.

### 129.2 El alcance real: migracion de formato en disco

49-A toca **dos rutas de serializacion independientes** que comparten
`record_to_bytes`:

| ruta | escribe | lee |
|---|---|---|
| sled | `persistence.rs:531` | `:309` |
| **snapshot** | `snapshot.rs:177` | **`:333`** |

⚠️ **La linea 333 lleva `take(48, "registro")`: la longitud esta codificada
en la llamada.** Un parche que solo tocara `store.rs` dejaria el snapshot
**escribiendo 80 y leyendo 48**.

**Formato**: `[u8; 48]` fijo, **sin byte de version**. Añadir `view_id` lo
lleva a 80 e **invalida toda cuenta ya escrita**.

✅ **Y la longitud fija es la herramienta, no el obstaculo**: `len()==48` ⇒
viejo, `==80` ⇒ nuevo. **Discriminador por longitud**, con precedente en el
arbol `legacy_null` que `persistence.rs` **ya migro**.

### 129.3 Decision declarada: NO-RETROACTIVA

Las cuentas pre-49-A cargan `view_id` = **centinela** —digest cero, que
`view_id_of` **nunca produce** porque mezcla el dominio en el estado—. Su
vista autenticada devuelve siempre `None`.

> **Misma familia que §117 y §119**: el salt no protegio las hojas viejas, la
> reversion no alcanzo los pendientes viejos, y la vista no protege las
> cuentas viejas. **Las tres lo eligen y las tres lo dicen.**

### 129.4 ⚠️ El borrador retirado, y su punto (c)

El primer intento fue un heredoc ciego que **(a)** llamaba a una funcion
**inexistente** mientras preguntaba si existia; **(b)** rompia §90 con
`spend_key[0]`; **(c)** dejaba **las dos rutas de snapshot sin tocar**.

> **El (c) es el que asusta: escribir 48 por una ruta y leer 80 por otra NO
> da error de compilacion.** El fallo aparece al restaurar un snapshot.

**Lo cazo el rito de leer antes** — los greps que encontraron el segundo
consumidor y el constructor de `persistence.rs:316`.

⚠️ **La regla que sale de aqui**: **superficie publica viva se despliega por
pasos compilables en la maquina real, nunca de una tacada.** Es §97.1 otra
vez —el `regex` que hizo 18 cambios sin verse—.

### 129.5 El plan, en cinco pasos verificables

1. **Frontera de serializacion** (`store.rs`): `_v2` con formato dual por
   longitud + centinela; shims viejos `#[deprecated]` como **mapa exacto** de
   lo que falta migrar. **Diseñado y probado que aplica limpio; NO aplicado.**
2. **Estructura y apertura**: `AccountRecord` gana `view_id`;
   `open_with_id` lo recibe **como parametro** —no puede derivarlo: solo
   tiene la identidad (§93.4)—. ⚠️ **La via ancha debe resolver §90 de
   verdad**, no con `spend_key[0]`.
3. **Migrar los 5 call-sites**. Sub-decision abierta: el `take(48)` de
   `snapshot.rs:333` — ¿los snapshots viejos **se migran o se regeneran**?
4. **Puerta autenticada** en `client.rs`. Test: el rojo pasa; **los 97 usos
   de `balance_of` intactos**.
5. **Eliminar los shims** cuando el paso 3 los deje sin uso.

## 130. El canon de tiempos, re-medido con protocolo

⚠️ **Esta seccion se escribio DESPUES de §131-§133**, al descubrirse que
**seis citas de `AUDITORIA.md` y una del README apuntaban a una seccion que
no existia** — el canon vivia solo en el README y en el informe de sesion.

> Es **§120 otra vez** —una cita que el lector no puede seguir— y **la
> primera vez que muerde la laguna de la entrada 66**: el guardian valida
> secciones citadas **desde `.rs`**, y estas se citaban **desde los `.md`**.

Instrumento: `metrics.rs`, `mod remedicion_89_1`, protocolo §89.1 estricto —
**una ejecucion del proceso = una muestra**, cinco muestras, release.

### 130.1 El canon

| via | generacion | apply | prueba |
|---|---|---|---|
| **send** | **353,2 ms** | **36,4 ms** | **64.722 B** |
| **claim** | **243,1 ms** (caliente 236,9) | **38,1 ms** | **65.250 B** |

**Un pago completo ≈ 664 ms** —590 de generacion + 74 de `apply`—. Mil
transferencias: **~590 s y ~124 MiB**.

⚠️ **Cifras de UN contexto.** §131 mide la deriva entre sesiones en **~9 %**
y fija como se publican: **rango, no cifra con σ**.

### 130.2 Cuatro correcciones a la familia de numeros

1. **El instrumento no era ruidoso: era sensible al protocolo.** Con
   proceso-por-muestra dispersa **0,5-0,6 %**. Los 375 y 315 de t5b —bucles
   entrelazados, cache compartida— y el 353 de aqui **no son tres medidas de
   lo mismo: son tres protocolos**.
2. ⚠️ **El 620 canonico pasa a historico.** Estaba un **75 % por encima**.
   Recorria README, PRINCIPIOS y ESCALADO **y nadie sabe de donde salio** —
   se **retira una cifra huerfana**, no se corrige un error.
3. ⚠️ **«La prueba» son DOS numeros**: claim genera un **33 % mas barato**
   que send. El canon anterior trataba los circuitos **como uno**.
4. La dispersion del `apply` —4-6 %— contra la de la generacion —0,6 %— es
   **un dato**: el `apply` **lleva disco dentro**; la generacion es CPU pura.

### 130.3 Fase B: el efecto del digest, aislado con control

⚠️ **El primer diseño era invalido**: comparar el arbol nuevo contra la tanda
vieja **es la comparacion entre sesiones que §131.5 prohibe**. Rediseñado a
**protocolo APAREADO** —cinco pares nuevo/viejo alternados en una sesion, **la
deriva cancelandose en la resta**—.

| | |
|---|---|
| **Δ apply (viejo − nuevo)** | send **33,8 ± 4,1** · claim **35,3 ± 2,9 ms** |
| Prediccion registrada | 29,6 / 29,8 — **confirmada a 1-2σ** |
| Distancia de cero | **8σ** |
| ⚠️ **Control interno** | **Δ gen ≡ 0** (+4 ± 43) |

> **El control es lo que hace valida la medida**: la generacion **no lleva
> digest**, asi que su Δ nulo **mide el ruido ambiente y demuestra que el
> +34 del `apply` es efecto y no contexto**. Sin el, un Δ de +34 podria ser
> deriva.

### 130.4 ⚠️ Y §89.2 queda decodificado

`apply` viejo = **71,8 ± 1,7** en este contexto ⇒ el *«arbol y disco ~70 ms»*
de §89.2 era:

> **~60 ms de `digest_of_proof` SIN NOMBRAR + ~10-12 de verificar, arbol y
> disco.**

**§124 partio por la mitad el termino dominante del `apply`** — y **nadie lo
sabia cuando se hizo**: se justifico por la colision, no por el coste.

## 131. ⚠️ El instrumento tiene DOS dispersiones, y §130 publico una

§130 caracterizo el instrumento con proceso-por-muestra y sello **σ 0,6 %**,
concluyendo que *«el instrumento nunca fue ruidoso — era sensible al
protocolo»*.

**La primera mitad es cierta. La segunda esta incompleta.**

### 131.1 Dos tandas, internamente consistentes, mutuamente incompatibles

| | §130 (n=5) | verificacion (n=5) | Δ |
|---|---|---|---|
| send gen | 353,2 ± 2,3 | **322,2 ± 1,8** | **−8,8 %** |
| claim gen | 237,0 | **217,7 ± 0,9** | **−8,1 %** |
| send apply | 36,4 | **32,4** | −11 % |

> **322 esta a TRECE sigmas de 353.** Las dos tandas tienen σ ≈ 0,5 % dentro
> y **~9 % entre ellas**.

### 131.2 Hay dos dispersiones, y son cosas distintas

| | valor | que mide |
|---|---|---|
| **Intra-tanda** | **σ 0,5 %** | el instrumento **es un reloj** |
| **Entre tandas** | **~9 %, sistematico** | **el estado de la maquina cambia entre sesiones** |

⚠️ **Publicar σ 0,6 % sugiere reproducibilidad al 0,6 %, y no la hay al 9 %.**

Es **§123.2 otra vez** —alli se publico rango porque variaba un 22 %— **y
esta vez con un instrumento mejor**: mejorar el instrumento no elimino el
problema, **lo movio un nivel**.

### 131.3 El veredicto de §130, aplicado a §130

§130 sella que los numeros de t5b *«describen SU contexto, no el canon»*.

> **Los suyos tambien.** 375, 315, 353 y 322 no son cuatro errores ni cuatro
> protocolos: son **cuatro contextos**, y solo dos de ellos comparten
> instrumento.

**No lo tumba** —el canon nuevo sigue siendo el mejor que hay, y su tabla de
dos vias es un hallazgo real: **claim genera un 33 % mas barato que send**—.
Lo que cambia es **como se publica**.

### 131.4 Lo que se corrige en el README

De cifra con σ a **rango con las dos dispersiones declaradas**:### 131.5 ⚠️ Lo que NO se sabe, y no se supone

**Que causa la deriva del 9 %**: termico, turbo, cache, carga del sistema.
**No se investiga aqui y no se conjetura** — es la siguiente medicion.

⚠️ **Y una consecuencia que alcanza a todo lo de hoy**: los **10,84 s** del
diccionario (§93.2), los **28 ms** del digest (§124.2) y los **160 ms** de
XMSS (§114) **arrastran esa misma deriva** si se comparan entre sesiones.
Ninguno se invalida; **ninguno es comparable con una medida de otro dia al
9 %**.

### 131.6 La leccion, en una linea

> **Un instrumento preciso no es un instrumento exacto.** σ 0,5 % dice lo
> bien que el aparato repite; **no dice nada de si mide lo mismo mañana**.

## 132. B13/B14: diseñada y descompuesta. NINGUN AIR abierto

⚠️ **Numeracion**: el asiento llego como «§131», ya ocupada por la
caracterizacion del instrumento. Va como **§132**.

### 132.1 Por que no se abre a ciegas

> En restricciones de AIR, **un desajuste no falla al compilar: acepta
> pruebas que no deberia**.

Abrir ocho seria **el borrador de 49-A (§129) elevado a la octava
potencia**, en el unico terreno donde los errores son **invisibles**.

### 132.2 ⚠️ Cuatro correcciones del Paso 0 a la ficha de la entrada 50

| decia | el arbol dice |
|---|---|
| «coste **negativo** (§86)» | ⚠️ **FALSO**: §86 midio **ensanchar** —mas datos en merges **existentes**—. El salt **AÑADE un merge**. Coste **positivo** |
| «cinco circuitos» | **OCHO** computan `native_leaf`: burn, audit, mint, send, recovery, recovery_climb, claim, mint_climb —settlement son helpers, no AIR— |
| «un merge mas» | **estado nuevo en una maquina de DOS CARRILES** —`C_LEAF_CAP/DIG/NONCE` sobre lanes A y B con flag `link_leaf`—, replicado en **ocho geometrias distintas** |
| «clase entrada 15» | el **arbol mixto** es un problema de diseño **que la entrada 50 no tenia resuelto** |

⚠️ **La primera es §95.2, y la cita es mia**: se invoco «§86, coste negativo»
**tres veces hoy** sin comprobar que §86 midiera lo mismo. **Cabe en la
longitud** —`TRACE_LENGTH` 1024, la hoja llega a la fila 15— pero
`TRACE_WIDTH` 52 **crece ~16 columnas**. La cifra real la dara el
instrumento de §130 **cuando el circuito exista**, no la cita.

### 132.3 El arbol mixto, y la arquitectura elegida

Las cuentas viejas **no pueden saltearse del lado del servidor**: el salt
deriva de la clave y la capa **no la tiene** (§93.4, a proposito).

| salida | veredicto |
|---|---|
| Dos formulas con **selector del probador** | ❌ **carga de solidez real**: habria que probar que nadie abre una hoja vieja **como salteada** ni al reves |
| **Envoltura uniforme salt-cero** | ✅ **elegida** |

La capa **si conoce** `(id, balance, nonce)` de cada cuenta vieja ⇒ recomputa
sus hojas **una vez** como `merge(hoja_vieja, [0;4])`.

> **Arbol uniforme, UNA sola formula en los ocho AIR** —un merge, sin
> selector, **sin solidez condicional**— y la **no-retroactividad de §126.4
> cae POR CONSTRUCCION**: salt cero es publico, esas cuentas siguen
> barribles, **exactamente lo declarado**.

Misma familia que el centinela de 49-A. Y `derive_leaf_salt` **nunca produce
cero** —mezcla el dominio—, con assert que lo fija.

### 132.4 El prerrequisito: el paso 1 NO es tocar un AIR

Recomputar las hojas viejas **cambia la raiz del arbol de cuentas** ⇒ es un
**evento de migracion** que necesita `OpKind` propio, **prueba de que la raiz
nueva deriva legitimamente de la vieja**, y coordinacion con snapshots.

**Eso solo es una entrada.**

### 132.5 Descomposicion, ninguno abierto

| paso | |
|---|---|
| 0 | ✅ **Anatomia leida** — el arnes `mutation::{buscar_vacias, rows_of}` **ya existe** |
| 1 | **Migracion de raiz** (prerrequisito) |
| 2 | **Piloto en `circuit_send`**, con **test de MUTACION obligatorio**: sabotear el salt y ver que el AIR **rechaza** — **sin el, la restriccion es fe** |
| 3 | Replicar a los siete, **uno a uno, cada uno con su mutacion** |
| 4 | **T2b-circuito** (§126) |
| 5 | **Medir** el coste **positivo** con el instrumento de §130 |

### 132.6 ⚠️ La mitigacion provisional: una mitad vale, la otra se descarta

**Vale**: documentar en la apertura —mecanismo `#[deprecated]` que
`open_account` **ya usa** para el riesgo de 64 bits— que **las cuentas
abiertas hoy quedan barribles hasta el despliegue**.

⚠️ **Se descarta**: *«fundar la hoja con entropia en el balance/nonce
inicial»*.

> **El balance es dinero**: no se le puede meter entropia sin cambiar cuanto
> vale la cuenta. Y el nonce lo ata `C_NONCE` a **incrementos de uno**.

**No es una mitigacion pendiente: es una que no existe**, y registrarla como
pendiente habria sido deuda falsa.

## 133. La 67: NO aleatorizar la posicion — DERIVARLA

⚠️ **Numeracion**: el asiento llego como «§132», ya ocupada por B13/B14. Va
como **§133**.

### 133.1 ⚠️ «La mitigacion mas barata del proyecto» era lo contrario

Y el codigo lo dice en tres capas:

1. **El indice ES la coordenada en el arbol**, no una etiqueta:
   `set_leaf(index, ...)` con `index = next_index++`, y `path_for(index)`
   construye **toda prueba de pertenencia** sobre ella.
2. ⚠️ **El que mata la frase**: el cliente **no recupera su indice, lo
   recibe** —`account_view(index)`, `send(receiver_index)`—. Hoy es
   secuencial, asi que **un cliente que lo perdio puede encontrarse
   barriendo 0..n**.

   > **Aleatorizarlo cambia una fuga de privacidad por un MODO DE PERDIDA DE
   > FONDOS**: con solo su clave, nadie localiza su cuenta. **Rompe la
   > recuperacion que §127 acababa de construir.**
3. El snapshot **cuenta por densidad** —`accounts.len()`, lineas 171 y 210—.

**Es el cuarto §95.2 de la sesion, y los cuatro son mios**: «coste negativo»
(§132.2), «pone verde los tres» (§129.1), «un merge mas» (§132.2) y este.

### 133.2 La solucion arregla en vez de romper

**`posicion = H(public_id) mod 2^32`** —y el `public_id` ya deriva de la
clave (§117)—:

| | |
|---|---|
| **El atacante no elige vecino** | su posicion **la fija su identidad**, fuera de su control — **objetivo de la 67 cumplido** |
| **La recuperacion MEJORA** | un cliente con solo su clave computa `public_id → posicion` y **encuentra su cuenta sin barrer nada** |

> Cierra de paso el modo de perdida que la version aleatoria abriria, **y
> mejora sobre el barrido secuencial de hoy**.

✅ **Y el arbol YA es disperso**: `SparseMerkleTree`, `TREE_DEPTH = 32`,
capacidad 2^32. **El obstaculo que se temia no existe**: `public_id → [0,
2^32)` cabe nativo.

### 133.3 Por que es MEDIA y no barata

- ⚠️ **Colisiones**: dos `public_id` al mismo slot. Problema clasico con
  solucion conocida, **pero es decision de diseño con su prueba** — hay que
  **argumentar** que un atacante no puede forzar la coincidencia con una
  victima. Fabricar `public_id` cuesta 2^63 (§82), asi que es duro; **se
  argumenta, no se asume**.
- **Snapshot**: de contar por `len()` a serializar posiciones dispersas.
- **Migracion**: recomputa el arbol de cuentas.

### 133.4 ⚠️ La dependencia dura con §132 — y su riesgo

**Las dos migraciones recomputan el arbol**: la envoltura salt-cero de
B13/B14 y el reposicionamiento de la 67.

> Se hacen **en una sola pasada** o **se emiten dos eventos de migracion
> donde basta uno**.

⚠️ **Y hay que decir la otra mitad**: un evento que aplique **salt-cero Y
reposicionamiento a la vez** tiene **dos formas de salir mal y una sola
prueba de que salio bien**.

**Coordinarlas es mas barato y mas arriesgado.** La coordinacion necesita su
propio argumento de **por que un evento con dos cambios es verificable** —o
dos eventos consecutivos, mas caros y cada uno con su prueba—. **Sin
resolver.**

### 133.5 Descomposicion, ninguno abierto

| paso | |
|---|---|
| 0 | ✅ anatomia: indice = coordenada; cliente **recibe** el indice; arbol **disperso** 2^32; snapshot cuenta por `len()` |
| 1 | **Resolucion de colision con su argumento de solidez** — prerrequisito |
| 2 | Derivacion + **recuperacion por computo** |
| 3 | Snapshot disperso |
| 4 | **Coordinar con §132** — o justificar dos eventos (§133.4) |
| 5 | `account_indices_are_not_predictable` pasa a verde: **segundo de los tres rojos historicos** |

## 134. El orden, leido contra los asientos: cuatro anotaciones

El orden por fisica —mecanico, documental, diseño, condicionado, riesgo con
nombre, ruido— es solido como esqueleto. Leido contra el registro pide
**cuatro anotaciones**, y **dos cambian el orden en vez de matizarlo**.

### 134.1 La 52 es la clave de boveda, no un item

**Tres decisiones de esta sesion difirieron su coste a la rotacion:**

| | lo que difirio |
|---|---|
| §117 | «rotar la clave implica nueva hoja» |
| §127.5 | la clave de vista **es irrotable sin rotar la de gasto** |
| B18.2 | su reclamo de migracion **ES una rotacion con hoja nueva** |

Y la entrada 20 muestra que **medio espacio de diseño ya existe**: `recover`
**rota la clave de cuenta CON custodios** — el acoplamiento exacto que §128
dijo que hay que leer antes de proponer nada.

> **O se unifican los dos casos abiertos de la 20 + el reclamo de B18.2 + los
> tres costes diferidos en UN solo diseño de rotacion, o la rotacion se
> diseñara tres veces, incompatibles entre si.**

⚠️ **Y hay una restriccion estructural que debe leer primero quien la
diseñe**: la 20 registra que la rotacion de custodios es **por uso, no por
tiempo** —*«esta capa no tiene nocion de tiempo»*—. Es la misma razon por la
que §119.3 tuvo que anclar el reloj de la reversion a **cabezas firmadas**.
**La ausencia de tiempo ya ha condicionado dos diseños distintos.**

### 134.2 ⚠️ La 7 y B13/B14 colisionan en los mismos ocho AIR

La evidencia de la entrada 7 es que **§72 —una restriccion en el carril
equivocado— se habria cazado leyendo la spec, sin leer el codigo**.

Y la clase de bug que §132 mas teme del salt es **exactamente esa**: un
desajuste de constante de fila entre `place()` y la restriccion, **en dos
carriles**.

> **La seccion maquina-de-hoja de la spec —no la spec entera— sube a paso
> 1.5 de B13/B14**, y el piloto de `send` se escribe **contra** ella.
>
> **Especificar-modificar-completar, no modificar-y-luego-especificar-lo-
> modificado.**

### 134.3 La 42 se ata a la 32 — pero por su ficha, NO por el grep

**La anotacion acierta**: la 42 es `mint_to_pending`, **la via antigua**, y su
propia ficha lo dice —*«`apply_mint_pending_delegated` usa
`SupplyCapExceeded`; la antigua no se toco»*—. **Muere cuando la 32 retire
las vias.**

⚠️ **Pero la evidencia inicial era ambigua**: `client.rs:157` lanza
`OverRegulatoryLimit` **y es correcto** — es el limite regulatorio de una
**transferencia**, con su comentario citando §25.

> **Cuarta homonimia del proyecto** —tras los salts, los `receipt` y
> `native_merge`—: **el mismo `LayerError` significa cosas distintas segun
> quien lo lance**, y `grep` no las distingue.

**Y el prerrequisito de la 32 —medir el precio de la delegada— corre bajo
§130/§131**: proceso-por-muestra, **y apareado si compara vias**, o producira
otro numero-de-contexto disfrazado de dato.

### 134.4 Tres ausencias que el orden pierde en silencio

1. **La mitigacion provisional de §132.6** —documentar el riesgo en la
   apertura—: pequeña, separable, y **la unica respuesta inmediata a la
   urgencia que justifico poner B13/B14 arriba**.
2. **Los cinco pasos de 49-A**, con el paso 1 ya diseñado.
3. ⚠️ **Las migraciones de B13/B14 y de la 67 van ENCADENADAS** (§133.4). El
   orden debe mostrarlas asi **o alguien las ejecutara en serie y recomputara
   el arbol dos veces**.

### 134.5 Lo que el orden dice de la sesion

> **Ya nada entra por la etiqueta «barato».** Esta sesion la invirtio
> **cuatro veces** —§129, §130.2, §132.2, §133.1— y las cuatro etiquetas eran
> propias.

El triaje de §128 —mecanico / documental / diseño / condicionado— **se
convirtio en columna vertebral**, y el cubo de condicionadas (12, 62, 54)
esta bien aparcado.

## 135. Una edicion a un preprint publicado: hecha, revertida, declarada

⚠️ **Numeracion**: el asiento llego como «§134», ya ocupada. Va como **§135**.

### 135.1 La secuencia, completa

Al cerrar la 54 (§128.2) se **insertó** el parrafo «Changing the verifier»
en el cuerpo de `doc/preprints/ZK-SSL-residual-trust.md` §4.1 — **un
artefacto publicado con DOI**.

⚠️ **Y el asiento que lo narraba decia «revertido por el autor». El arbol
decia lo contrario**: el parrafo seguia ahi, y habia entrado en el commit
`337a434`, cuyo mensaje habla de **medicion de tiempos**.

> **Dos fallos en uno**: un artefacto publicado editado, **y** un commit que
> contiene mas de lo que dice (§91). El segundo es lo que hizo invisible al
> primero.

**Comprobado antes de registrar nada** — de otro modo se habria escrito una
reversion que no habia ocurrido.

### 135.2 El error no es de contenido: es de VEHICULO

El riesgo del verificador **es real y esta bien escrito**. Lo que estaba mal
es **por donde entro**.

⚠️ **Y la regla que lo prohibe se dicto en esta misma sesion, por la misma
mano**:

| | |
|---|---|
| §119 | reescribir el referente **lo falsifica** |
| §122 | los dos documentos recuperados van **verbatim**, con cabecera-mapa **encima**, no arreglados |
| §130 | la cascada de correcciones **excluyo los preprints explicitamente** |

**La regla existia, era propia, y no se aplico.**

### 135.3 Ejecutado: revertido, y con vehiculo nuevo

✅ **Revertido**: 783 bytes, 12 lineas, cero coincidencias. El artefacto
vuelve a estar **verbatim**.

✅ **`doc/preprints/ERRATA.md` creado** — el vehiculo **aditivo** que la
entrada 28 no tenia: **como corregir algo publicado sin reescribirlo ni
esperar a una revision**.

Y con lo que el fichero **declara que NO hace**:

> Un lector que descargue el PDF **no ve este fichero**. Registrar una errata
> **no la corrige para quien ya la leyo**.

### 135.4 ⚠️ Su entrada 2: el directorio incumplia su propia regla

`ERRATA.md` nace con **dos** entradas. La segunda reconoce que **las cuatro
notas de correccion de §100 se insertaron EN el cuerpo** de dos preprints.

**Su contenido es correcto. Su vehiculo era el que este fichero declara
equivocado**, y lo usaron antes de que el fichero existiera.

**No se deshacen**: ya estan depositadas, y retirarlas seria **reescribir el
artefacto otra vez — el mismo error en direccion contraria**.

> **Un fichero de erratas que declara una regla sin admitir que el directorio
> la incumple en cuatro sitios naceria falso.**

### 135.5 La leccion, en la columna de metodo

> **El registro debe contener las reversiones con el mismo rigor que los
> cambios**, o el metodo tiene un punto ciego **exactamente del tamaño de un
> `git revert` sin asiento**.

⚠️ Y una segunda, que este caso añade: **un asiento puede describir un estado
que el arbol no tiene**. Paso al metodo: **antes de registrar una accion,
comprobar en el arbol que ocurrio** — igual que §130 se descubrio ausente
porque seis citas la buscaban.

## 136. La mitigacion de §132.6, y una falsa alarma sobre una cifra propia

### 136.1 ✅ El aviso en la apertura, hecho

`accounts.rs` documenta en **las dos puertas** —`open_account` y
`open_account_wide`— que una cuenta abierta hoy **queda barrible hasta el
despliegue del salt**.

⚠️ **Y dice lo que no era obvio**: la fuga **no depende de la entropia de la
clave, sino de la ausencia de salt en la hoja** — asi que **las dos anchuras
son igual de vulnerables**.

> Sin esa frase, un lector supondria que `open_account_wide` protege mas
> **porque protege mas en otra cosa** —los 256 bits—. **Falsa sensacion de
> seguridad por asociacion.**

Y **no promete temporalidad**: *«no se corrige rotando ni esperando»*, que es
§126.4 dicho donde el titular lo lee.

### 136.2 ⚠️ Falsa alarma: los «10,84 s» SI estan anclados

Se reporto que la cifra del diccionario **no aparecia en `AUDITORIA.md`** —
citada toda la sesion, en §108, §117, §126, §131.5 y en `SECURITY.md`, sin
fuente canonica—. Habria sido peor que el caso de §130: alli faltaba una
seccion; aqui faltaria **una medicion publicada**.

**Es falso.** §93.2, en el registro:

> *«**Medido: saldo del vecino recuperado en 10,84 s.** 743.100 centimos,
> exacto.»*

Con su curva sobre el rango de saldo, sus priors y su alcance acotado.

⚠️ **Por que fallo la comprobacion**: el `grep` busco **las palabras que
describen el hallazgo** —«barrib», «diccionario»— y la cifra esta anclada
**por su numero**.

> **Una cifra bien anclada puede parecer huerfana si se la busca por su
> contexto en vez de por su valor.** Es **§99.5 en la herramienta de
> verificacion**: un dato correcto buscado sobre el objeto equivocado.

**La leccion es util aunque la alarma fuera falsa**: verificar la existencia
de un dato se hace **por el dato**, no por su narrativa.

### 136.3 Tres ayudantes de test sin usar

`circuit_settlement.rs` da tres avisos: `SK` y `d()` en
`t2b_recuperacion_nativa`, `claves()` en `t2a_salt_hoja` — **ayudantes que
sobrevivieron a la escritura de sus tests**.

⚠️ **Peso muerto, no garantia falsa** — a diferencia de los `assert_eq!`
tautologicos de §125.2, que **afirmaban algo**. Aqui no se retira nada
todavia: se **nombra**, para que la limpieza no se confunda con un hallazgo.

### 92.5 Lo que queda

**Los otros cuatro circuitos de gasto** —`send`, `claim`, `burn`, `audit`—
mas la derivacion compartida y `open_account`, en un commit. Y despues
**custodios y gobernanza**, acoplados por `build_custodian_set`, en otro
(§85.7).

El patron esta establecido: columnas, ranuras, grados, `build_trace`,
**aserciones** y tests. Y la lista de sitios que se olvidan, tambien.

## 137. 49-B: intentada, ROTA por una constante, revertida — y la pregunta de arquitectura cerrada por el fracaso

**Qué se intentó.** Posición derivada de la identidad para altas nuevas
(`public_id[0] mod capacidad`, sondeo lineal determinista, `next_index`
conservado como censo de cuota), sin migración: las cuentas viejas se
quedaban donde estaban (no-retro, familia §117/VIEW_ID_LEGACY) y la
migración única de B13/B14 haría después las dos cosas de una pasada. El
análisis previo verificó contigüidad (`0..next_index`: solo pendientes),
tests con índices literales (ninguno), iso (índice opaco) y la capacidad
del árbol de CUENTAS. El parche compiló y sus dos tests nuevos pasaron.

**Qué pasó.** La suite completa —no filtrada, la regla que existe para
esto— devolvió **27 rojos**: todo lo que congela, emite, recupera, envía
o transfiere. Reversión inmediata (`git checkout`), árbol de vuelta a
232/2. Los 27 no se parchearon: se revirtió la decisión.

**La causa raíz, verificada y no especulada: `FROZEN_DEPTH = 24`.**
El árbol de congelados tiene capacidad 2^24; el de cuentas, 2^32. El
índice de cuenta es **coordenada COMPARTIDA entre dos árboles de
capacidades distintas**: `frozen.path_for(sender_index)` vive dentro de
send (two_phase:248), claim (:481), burn, transfer y las vías delegadas.
La invariante implícita «índice < 2^24» la cumplía la asignación
secuencial —de gratis, sin declararla— y la violó la posición derivada
(índices hasta 2^32, 256× la capacidad del frozen). La doc de
`capacity()` lo advertía: *«quien asigne posiciones debe comprobar este
límite»* — se comprobó el límite de UN árbol; el índice cruzaba DOS.

**Lo que el fracaso cierra.** La pregunta «¿49-B sola o con B13/B14?»
tenía una respuesta que ninguna opción contemplaba: **sola es
estructuralmente imposible**, no ineficiente. Tres salidas, todas caras:
(1) encoger el espacio a 2^24 → la molienda para dirigir posición baja
de ~2^31 (horas) a ~2^23 (minutos): destripa la mitigación — NO;
(2) crecer `frozen` a 32 → cambia su raíz → es SU propia migración →
pertenece a la migración única, que pasa a migrar TRES cosas de una
pasada: reposicionar cuentas + salt-cero en hojas + frozen a profundidad
32 — y `circuit_freeze`/`circuit_burn` verifican caminos de 24, así que
**los AIR de congelación también cambian: B13/B14 crece**;
(3) desacoplar índice y slot de congelación (mapa aparte) → complejidad
nueva y un mapa más que persistir y probar — peor que (2).
**Veredicto: la (2).** 49-B queda CONDICIONADA a la migración única de
tres frentes. El «una pasada» de §133 no era preferencia de eficiencia:
era necesidad, y costó 27 rojos demostrarlo.

**Corrección al método, con nombre.** El análisis de contigüidad miró
`next_index`, literales e iteraciones — pero no censó **quién más usa el
índice como coordenada**. Regla nueva: *antes de cambiar el dominio de
un identificador, censar TODAS las estructuras que lo usan como
coordenada y comprobar el límite de CADA una* — el grep no es
`0..next_index`, es `path_for(.*index)` en todos los árboles.

**Lo que sobrevive del intento** (para la migración única): el diseño de
`derived_position` con sondeo (correcto sobre un frozen de 32), la cota
honesta de ~2^31, la corrección de que el snapshot ya es
disperso-compatible (serializa pares índice-registro; la objeción de
§133 era sobre-cautela), y los dos tests escritos, válidos entonces.
Nada de esto está en el árbol hoy: la suite está en 232/2 y 49-B sigue
roja, que es la imagen fiel.

## 138. Cuestión previa de la máquina de hoja: RESUELTA antes de tocar el salt (no explotable)

**Contexto.** La spec de la máquina de hoja (paso 1.5 de B13/B14) señaló,
antes de tocar ningún AIR, una duda de soundness: `C_NONCE` en
`circuit_send` (y `circuit_burn`, línea 592) ata SOLO `next[8]` en el
merge del nonce — `link_leaf * (next[8] - COL_NONCE)`. Los limbos `[9]`,
`[10]`, `[11]` del rate no tienen restricción propia. El nativo los pone
a cero (`as_digest(nonce) = [nonce,0,0,0]`), pero un probador construye
su trace: nada en las restricciones se lo OBLIGA. Tres líneas más abajo,
`C_KEY_INPUT` sí ata los cuatro, citando §92.2 — la lección estaba en la
clave y NO en el nonce.

**Método: escribir el test que DECIDE, no suponer.** Calcando el patrón
de mutación de `a_send_with_inconsistent_receiver_identity_is_rejected`:
construir un trace válido, envenenar `[9]` en `ROW_LEAF_LINK`, y probar
de extremo a extremo (`prove` -> `verify`). Más un CANARIO (envenenar
`[8]`, el limbo atado) para descartar que la mutación no llegara al
circuito — un test que pasa por la razón equivocada es peor que ninguno.

**Resultado: RECHAZA. No explotable.** Ambos en verde:
`un_nonce_con_limbo_alto_no_cero_se_rechaza` (el limbo alto corrupto no
verifica) y `canario_el_limbo_atado_del_nonce_tambien_rechaza` (el limbo
atado también rechaza -> la mutación en ROW_LEAF_LINK SÍ llega; el primer
test no es vacío).

**El porqué (leído en código, no inferido).** `C_HASH_A/B` (línea 804)
ata la permutación Rescue COMPLETA — las 12 columnas del estado, incluido
el rate `8..12`, en toda fila con `hash_flag=1`. En `ROW_LEAF_LINK`,
`link_leaf` fija el rate del estado siguiente con el nonce; la fila
siguiente tiene `hash_flag=1`, y `C_HASH` exige que ese estado completo
(rate incluido) sea la preimagen correcta de la permutación. Un `[9]`
corrupto entra en el MDS de la ronda siguiente y rompe `C_HASH` aguas
abajo. **`C_NONCE` no necesita atar los cuatro limbos: la cadena de
hasheo los arrastra.** El limbo suelto es redundancia cubierta, no un
agujero.

**Consecuencia para B13/B14.** El suelo del piloto aguanta: el merge del
nonce es sólido, y el salt se añade sobre terreno probado. Para
`C_SALT_IN` (delta salt de la spec) se elige atar los cuatro limbos —como
la clave— por claridad y defensa en profundidad, sabiendo ahora que
C_HASH ya cubriría: cinturón sobre tirantes, sin coste de soundness.
`circuit_burn` tiene el patrón idéntico; su test de mutación, al migrarlo
(paso 3), debe llevar el mismo canario — no se asume que lo de send vale
para burn sin probarlo allí. Commit `1ba848d`.

## 139. El piloto del salt: TRES reversiones que mapearon el obstáculo (circuit_send codifica cada posición en tres sitios)

**Qué se intentó.** Insertar el tercer merge del salt en `circuit_send`
(SB1 del plan del piloto: reindexado puro, restricciones inertes) para que
la hoja pasara a `native_leaf_salted`. La spec de la máquina de hoja lo
planteó como corrimiento +1 ciclo de los tramos posteriores al salt.

**Qué pasó: tres reversiones, cada una destapando una representación.** El
trace codifica la posición de cada camino de Merkle (cuentas, frozen,
pending) en TRES sitios que ninguna edición parcial mantiene en sincronía:
(1) las constantes `ROW_*` en filas (`ROW_ROOT=271`, `ROW_FROZEN_ROOT=471`
…); (2) los offsets de ciclo de las columnas, `(2 + level)*CYCLE_LENGTH`
cuentas / `(35 + level)` frozen / `(61 + level)` pending, en los bucles de
`build_trace` Y en las periódicas `link_merkle`/`frozen_link`/`pend_link`;
(3) los rangos + aritmética del `match r` (líneas 477-489), que calcula el
nivel del path con rangos HARDCODEADOS y offset distinto por tramo:
`(2..34) -> next_cycle-2`, `(36..60) -> next_cycle-35`, `(62..94) ->
next_cycle-61`.

Intento 1 (solo `ROW_*` +8): rompió (offsets de ciclo viejos). Intento 2
(`ROW_*` +8 Y offsets +1): rompió IGUAL — el `match` seguía con sus rangos
viejos, `place_frozen` recibió `level=24` sobre un path de 24 (índice
fuera de rango, misma línea las dos veces). Reversión `git checkout` cada
vez; árbol a 286/0 intacto.

**El hallazgo (método, no código).** `circuit_send` es demasiado
intrincado para modificar su geometría por corrimiento manual: cada
posición vive en tres representaciones con convenciones de off-by-one
DISTINTAS, y los rangos del `match` tienen ciclos-frontera entre tramos
(saltos 34→36, 60→62) que no son uniformes. **Tercera aparición del
fenómeno de §137**: censar TODAS las representaciones de una coordenada,
no la muestra. El grep de censo buscó `const ROW_` y `* CYCLE_LENGTH`; se
perdió los rangos del `match`. Regla reforzada: antes de mover una
coordenada en un circuito, grep de las TRES formas.

**Estrategias para la próxima sesión (a decidir con el mapa completo, NO a
ojo).** HACK: el salt en la holgura del final (filas 744→751, 281 libres),
enlace lógico apuntando allí, cero corrimiento — ninguna representación se
toca; coste: el trace deja de leerse en orden temporal. REFACTOR: unificar
las tres representaciones (derivar offsets y rangos de las constantes
`ROW_`), luego insertar — correcto pero sesión propia. **Recomendación:
producir primero el MAPA COMPLETO de la geometría de `circuit_send` como
documento; con él la elección es obvia y la ejecución deja de tropezar.
Sin él, un cuarto intento repetiría el patrón.**

**Imagen fiel.** El piloto NO avanzó; el árbol quedó 286/0 y 240/2, sin
rastro de los intentos. Lo que avanzó es el conocimiento del obstáculo —
imprescindible: sin estas tres reversiones, quien abriera el piloto habría
caído en la misma trampa. El plan SB1→SB5 sigue válido salvo la mecánica
de dónde meter las 8 filas, que es lo que se replantea.

## 140. El mapa de la geometría de circuit_send: producido, verificado mecánicamente, y la elección tomada — REFACTOR (SB0)

**Qué existe.** El mapa que §139 mandó producir antes de decidir:
`doc/mapa-geometria-circuit_send.md` (calendario completo, las tres
representaciones sitio por sitio con líneas de `1cbfedc`, tabla de
guardianes de frontera, censo ampliado, presupuesto, plan). Y su
verificador: `tools/verifica_geometria.py`, que no confía — reconstruye
el calendario fila a fila desde las constantes exactas, cruza las tres
representaciones entre sí y reproduce los intentos 1 y 2 de §139.
Ejecutado en esta máquina: secciones A–F en verde; los dos intentos
estallan en `place_frozen: nivel 24 sobre path de 24 (fila 471)` — la
misma línea que costó tres reversiones encontrar.

**Lo que el mapa encontró que §139 no tenía.**
1. **Los guardianes de frontera son de TRES tipos** — el límite del
   propio rango, el sombreado por un brazo explícito, y el límite del
   bucle — y solo el primero viaja con el rango. El valor ilegal de
   frozen (`nc=59` → nivel 24) lo impide SOLO el brazo de
   `ROW_FROZEN_ROOT`; el de pendientes (`nc=93` → nivel 32) lo impide
   SOLO el límite del bucle (`ROW_PENDING_ROOT` no tiene brazo). Por eso
   ambos intentos detonaron precisamente en frozen: era el único tramo
   cuyo guardián era la constante que el corrimiento movía.
2. **La forma derivada existe y es exacta**: las nueve `ROW_*` colapsan
   en ocho arranques de ciclo `CYC_*` derivados de `(CYCLE_LENGTH,
   TREE_DEPTH, FROZEN_DEPTH)` — verificado dígito a dígito (sección F).
   Normalizar los rangos del `match` a la convención única toca solo
   valores hoy inalcanzables: re-expresión byte-idéntica.
3. **`FROZEN_DEPTH` es coordenada compartida** (`circuit_freeze.rs:61` →
   send/claim/burn/frozen_climb + settlement; la capa ya tiene
   `FROZEN_DEPTH_POST=32` de 1b). El flip a 32 no puede ser solo-send
   vía la constante. Cuestión de estadificación ABIERTA (mapa §7, dos
   opciones), decisión del autor; bloquea la parte frozen del piloto y
   el paso 3 — NO bloquea SB0.
4. Correcciones de registro: la holgura son **280 filas** (744..1023),
   no 281 — la 743 sostiene las raíces atadas por aserción; el
   comentario de `TRACE_LENGTH` («llegan a la 1007… 16 de margen»)
   describe una geometría muerta; §139 citó líneas de un estado
   intermedio (el `match` vive en 464-478 en `1cbfedc`).

**Presupuesto verificado (sección E).** Salt +8 y frozen-32 +64 llevan
`ROW_PENDING_ROOT` de 743 a 815; margen 208. `TRACE_LENGTH=1024`
alcanza para send — primera fila con dato de la tabla de la spec §3.

**La elección: REFACTOR (SB0), luego el salt.** El argumento no es de
gusto: frozen-32 obliga al corrimiento (mueve `ROW_FROZEN_ROOT` — fila
de aserción y de siembra — y todo lo posterior) que el hack pretendía
evitar. El hack solo aplazaría 8 de las 72 filas y cobraría 12-20
columnas de transporte, partir `tree_link`, y una traza no temporal
multiplicada por los diez AIR del paso 3. SB0 va en pasos compilables
(regla §129), cada uno con suite verde y guarda md5: SB0.1 bloque
`CYC_*` + `ROW_*` derivadas con clavos `const _: () = assert!(…)`;
SB0.2 los literales 2/35/61 de los seis bucles; SB0.3 rangos del
`match` normalizados + `debug_assert!` en los tres `place_*`; SB0.4
comentarios, clavos fuera, frase de convención. Tras eso, SB1 vuelve a
ser lo que la spec diseñó y su mecánica de corrimiento es una línea. La
cuarta reversión que §139 temía se evita no eligiendo mejor entre dos
maneras de mover tres representaciones, sino dejando de tener tres.

**Nota de instrumento.** Al poner la baseline de esta sesión, la suite
se corrió sin `--release` por error del asistente: en debug, stark da
282/0 con 6 ignorados y zk-ssl **147/86** con 12. Los totales cuadran
exactos con los 288 y 245 declarados y con los ignores condicionados a
debug (cada uno con su motivo en el código), pero los 86 caídos de
debug no están censados en ningún sitio. La suite canónica es en
release (README:251-252; «31 s en release»). La causa de la caída en
debug no se conjetura (131.5); si algún día importa ese modo, se censa
entonces.

## 141. SB0 ejecutado completo: circuit_send con UNA representación de su geometría (cuatro pasos, suite verde en cada eslabón)

**Qué se hizo.** El refactor que §140 eligió, en los cuatro pasos
compilables del plan (regla §129), cada uno con guarda md5 atómica,
suite release canónica (286/0 + 2 ignorados en stark, 240/2 + 3 en
zk-ssl; los 2 rojos = 49-B) y commit propio:

- **SB0.1** (`0451462`): bloque `CYC_*` derivado de `(CYCLE_LENGTH,
  TREE_DEPTH, FROZEN_DEPTH)`; las nueve `ROW_*` re-expresadas como
  derivadas; nueve clavos `const _: () = assert!(…)` con los valores
  heredados — **el compilador atestiguó la identidad, no esta sesión**.
- **SB0.2** (`564f45e`): los seis bucles (tres de bits, tres
  periódicas) sobre `CYC_ACC`/`CYC_FROZEN`/`CYC_PEND_CLIMB`; cero
  literales de ciclo en bucles.
- **SB0.3** (`0639a52`): el `match` normalizado a la convención única
  `(CYC_arranque..CYC_fin)` con `nivel = nc − CYC_arranque` — frozen
  (36..60)→(35..59), pendientes (62..94)→(61..93). Los cuatro valores
  que cambian eran inalcanzables (sombreado / límite del bucle, mapa
  §3), y ahora **el rango se guarda su propio valor ilegal superior**
  en los dos tramos que antes dependían de guardianes externos — el
  defecto exacto que detonó los intentos de §139. `debug_assert!` en
  los tres `place_*`: el próximo desajuste dirá «nivel X sobre path de
  Y» en vez de estallar por índice.
- **SB0.4** (`96264ba`): comentarios al marco único (el muerto del
  margen «1007/16» fuera; el pendiente en el marco de hasheo, como su
  vecino); frase de convención junto al bloque; los nueve clavos
  retirados y en su lugar UNA guarda de relación:
  `const _: () = assert!(ROW_PENDING_ROOT < TRACE_LENGTH)` — el
  presupuesto de la traza en compilación, la que avisará con salt y
  frozen-32.

Cadena md5 de `circuit_send.rs`: `1b7110b9…` → `77f88cb4…` →
`77771824…` → `c0d4cce6…` → `0573f724…`. La cuarta reversión que §139
temía no ocurrió: no quedaba nada que desincronizar.

**Hallazgo colateral, censado para el paso 3.** Los rangos literales
del `match` viven en NUEVE ficheros hermanos: `circuit_claim` con los
tres tramos, `circuit_burn` con dos (cuentas + frozen), y siete con
uno (cuentas): `circuit_audit`, `circuit_mint`, `circuit_mint_climb`,
`circuit_recovery`, `circuit_recovery_climb`, `circuit_settlement`,
`double_entry`. La réplica del patrón SB0 tiene su lista hecha antes
de empezar — y cada uno se censa al migrarlo (§138: no se asume que lo
de send vale sin probarlo allí).

**Lo que queda listo.** SB1 (salt) vuelve a ser lo que la spec diseñó:
la mecánica de corrimiento es añadir `CYC_SALT` y correr `CYC_ACC` una
línea; el trabajo real son las columnas testigo, `link_salt`, las seis
`C_SALT_*` (cuatro limbos atados, §138) y las 2 mutaciones
obligatorias. La estadificación de `FROZEN_DEPTH` (§140.3) sigue
ABIERTA, decisión del autor; bloquea la parte frozen del piloto, no el
salt.

## 142. Estrategia del piloto: la costura con la capa obligaba a elegir — GEMELO ANDAMIO (C), y el gemelo ya nació

**La bifurcación que el mapa declaró fuera de alcance (mapa §9).** D4
de la spec de migración manda: los AIR nuevos verifican el mundo nuevo
y no el viejo; **circuitos + capa + migración van en UNA release**.
Censo de la costura: la capa construye la traza de send en dos sitios
(`two_phase.rs:260`, `client.rs:285`) con el camino sacado de su árbol
actual, de hojas sin envolver — si `circuit_send` computara la hoja
envuelta en el sitio, esa hoja subiría con hermanos del árbol viejo y
la raíz no sería la del ledger: caería todo test de capa que ejercite
un envío (el envío vive en ≥6 ficheros — two_phase, tests, client,
iso, metrics, tests_support — con 57 referencias; el conteo exacto de
rojos solo lo daría correr, pero son decenas, no dos).

**Las dos estrategias, costeadas.** (A) En el sitio, con interregno
rojo: stark verde y autocontenido, la capa compila pero decenas de sus
tests quedan «correctamente rojos hasta el flip» durante toda la
campaña del paso 3 — y el precio real es que **el semáforo deja de
informar**: un rojo nuevo de verdad se camufla en la pared; el
precedente de la casa son dos o tres rojos nominales, no una pared.
(C) Gemelo como andamio declarado con retirada en el flip (precedente:
los shims `#[deprecated]` de 49-A, aceptados y retirados a tiempo): el
piloto entero vive en la copia, la capa no se toca y sigue 240/2 toda
la campaña, el paso 3 replica el patrón, y el flip es exactamente la
release única de D4 — la capa cambia imports, los legacy se borran, la
fixture legacy prueba la migración. Coste de C: duplicación temporal
(~2.100 líneas por gemelo) contra §125, mitigada por ser andamio
DECLARADO con cláusula de retirada escrita en el propio módulo.

**Ratificado por el autor: C.** Y C paga un bonus solo: la
estadificación de `FROZEN_DEPTH` (§140.3) queda **RESUELTA de facto**
por la vía (i) — el gemelo llevará profundidad frozen local a 32 sin
tocar la constante compartida de cinco circuitos, y el flip unifica.

**SB1.a ejecutado** (`bab7097`): nace
`crates/stark-experiment/src/circuit_send_salted.rs` como copia
compilante de `circuit_send` (md5 del gemelo `7af238ed…`; `lib.rs`
`5568fcd5…` → `ad708f9a…`), con la cabecera-andamio y la cláusula de
retirada en el código. El guardián de ranuras lo absorbió sin
presentaciones (27 → **28 circuitos**); suite stark 286 → **305/0**
(+19 del gemelo), capa intacta 240/2. El rebanado que sigue, cada
eslabón con su rito: **SB1.b** geometría (`CYC_SALT`) + columnas
(`COL_LEAF_SALT`, 52→56) + brazo de siembra + `scenario()` al mundo
envuelto; **SB1.c** `link_salt` + las seis `C_SALT_*` (24 ranuras,
grados, evaluate); **SB1.d** las dos mutaciones de la spec §4 + test
nativo↔circuito; **SB1.e** frozen-32 local con la primera fila real de
la tabla de presupuestos.

## 143. SB1 COMPLETO: el piloto send vive entero en el gemelo — salt + frozen-32 + mutaciones que muerden

**Los cinco eslabones**, cada uno con guarda md5 atómica, suite release
canónica y commit propio (cadena del gemelo: `7af238ed…` → `1de69a2f…`
→ `3cb26323…` → `fccaa41d…` → `e239fc5b…`):

- **SB1.a** (`bab7097`, §142): nace el gemelo; el guardián de ranuras
  lo absorbe (28 circuitos).
- **SB1.b** (`b9f8a74`): el mundo envuelto — `CYC_SALT` entra en la
  cadena derivada y TODO el calendario se corre solo (el dividendo de
  SB0, cobrado); 4 columnas `COL_LEAF_SALT` (52→56); brazo del tercer
  merge; `scenario()` con el salt REAL (`derive_leaf_salt_wide(key)`,
  §117). Los 19 tests pasaron A LA PRIMERA en el mundo nuevo, hitos
  contra el nativo incluidos.
- **SB1.c** (`8066497`): las seis `C_SALT_*` (24 ranuras) cosidas por
  `link_salt`/`P_LINK_SALT`, grados y evaluate; el arnés de vacuidad
  las cubre: NINGUNA decorativa. El hueco de la fila 15, cerrado.
- **SB1.d** (`0549016`): las DOS mutaciones de la spec §4 RECHAZAN —
  (a) limbo del salt testigo alterado (veneno = honesto+1, dispara
  `C_SALT_IN`); (b) la hoja sin envolver al camino (dispara `C_PLACE`
  en el enlace corrido) — más el nativo↔circuito: la cadena de tres
  merges espeja `native_leaf_salted` limbo a limbo, ambos carriles.
- **SB1.e** (`7f84ff8`): frozen-32 LOCAL — el gemelo declara su
  `FROZEN_DEPTH = 32` y deja de importar la compartida (§140.3 opción
  i, materializada): el flip aquí es borrar la const y devolver el
  import. `frozen_climb_32` en tests, con cláusula de retirada.

**Presupuesto MEDIDO** (primera fila de la tabla de la spec §3,
rellenada en `doc/`): send 743 → +salt 751 → +frozen-32 **815**;
holgura **208 filas** (26 ciclos). `TRACE_LENGTH = 1024` alcanza — y
lo jura la guarda de SB0.4 en compilación, no este asiento.

**Dos hallazgos para el checklist del flip.**
1. `frozen_climb` (`circuit_freeze.rs:166`) **clava la profundidad por
   dentro** (`0..FROZEN_DEPTH`), no itera el camino: con path de 32
   subiría 24 y callaría. Al flip: o la compartida pasa a iterar el
   camino, o su constante sube con los cinco — decisión de ese paso,
   aquí anotada.
2. **La cuarta representación mordió al asistente**: SB1.b corrió el
   calendario y dejó desfasados los números de los comentarios del
   gemelo (743/488/280…) — el fenómeno exacto de §140.4, en la mano
   que lo había cazado. SB1.e los repuso anclados a nombres. Regla
   viva: los números de comentario se revisan EN el paso que mueve la
   geometría, no después.

**Qué prueba y qué no.** El checklist de la spec para send está ENTERO
(salt + frozen-32 + mutaciones + nativo↔circuito) y la capa siguió
240/2 toda la campaña — la promesa de la estrategia C, cumplida. NO
prueba: los otros nueve AIR (paso 3, con la lista de rangos de §141 y
las variantes por circuito que la spec §5 advierte: mint sin carril de
resta, audit sin mutar estado…), T2b-circuito (paso 4), la medición
apareada (paso 5), ni el flip (release única D4: capa + gemelos +
migración, y los legacy se borran).

## 144. Claim COMPLETO en el gemelo — el playbook replica (dos de diez)

**Los seis eslabones** (`e0986fb`→`29a8d81`→`ba2a380`→`d9d8786`→
`efbe0e3`→`a9e178e`; md5 del gemelo `603ff086…`→`35092bb4…`→
`c62f7487…`→`22aa8a03…`→`4f872dfa…`→`522da3e3…`), cada uno con guarda
atómica y suite release canónica: **R1** nacimiento (guardián 28→29);
**R2** SB0 interno FUSIONADO — la identidad de las nueve derivadas
verificada mecánicamente ANTES de escribir (el sustituto del testigo-
compilador cuando el parche es uno); **R3** mundo envuelto (calendario
a 751 por derivación, «EL SALDO SUBE» conservado, los 18 tests a la
primera); **R4** las seis `C_SALT_*` — y claim TRAE arnés de vacuidad:
las 24 quedaron cubiertas dinámicamente en el mismo verde; **R5** las
dos mutaciones de la spec §4 RECHAZAN + espejo nativo↔circuito con el
carril B sumando; **R6** frozen-32 local (815, holgura 208, jurado en
compilación).

**Lo que claim enseñó al playbook.** (1) El R2 fusionado funciona: un
parche compilable con verificación de valores pre-escritura sustituye
a los cuatro pasos del piloto sin perder garantía. (2) Las variantes
entran solas si el censo R0 las nombra: `escenario_para`/`destinatario`
(el parámetro nacido de cazar §39) intacto, la suma del receptor
conservada, la colisión `COL_SALT` resuelta por convención
(`COL_LEAF_SALT = 51`, ancho 55). (3) **La ventana R3→R6 de los
comentarios**: R3 movió geometría y los números de comentario quedaron
desfasados UNA ventana de commit hasta que R6 los llevó a los finales
— la forma leve del fenómeno de §143.2; para los siete restantes, R3
lleva la renumeración consigo o la declara diferida a R6 en su commit.

**Presupuesto MEDIDO** (segunda fila de la tabla §3): claim 743 → 751
→ **815**; holgura **208** (26 ciclos). CABE.

**Estado**: dos gemelos de diez (send §143, claim §144); capa 240/2
toda la campaña. SIGUE: **burn** (cuentas + frozen, dos tramos — el
último con fase frozen antes de los siete de un tramo), por su R0.

## 145. Burn COMPLETO — el primer desborde, cazado por la tabla y resuelto con el dato

**El hallazgo que justifica la campaña de presupuestos.** El censo R0
dio `TRACE_LENGTH = 512` con la tubería en 471 — holgura 40 — y la
cuenta del mundo nuevo dio: +salt 479, +frozen-32 **543 ≥ 512. NO
CABE.** Primer desborde de los diez, exactamente el caso para el que
la spec §3 exigió la tabla («no se asume que 1024 alcanza» — aquí fue
512 el que no alcanzó). La decisión, con el dato delante: compactar no
existe (hoja + subida + identidad + frozen, nada plegable sin
rediseño) → **TRACE propia a 1024**, la potencia siguiente, holgura
final 480 (60 ciclos), jurada por la guarda en compilación. El coste
~2× ya asoma en el instrumento: la suite stark pasó de ~13,9 s a
~15,0-15,6 s con burn probando a 1024 — dato crudo para la medición
apareada del paso 5.

**Los cinco commits** (`8a0da2b`→`71b0ccd`→`9cea05b`→`40be1d8`→
`48b0ef9`; md5 del gemelo `30c15ff3…`→`1c5401de…`→`d68065a9…`→
`bfd3ed4d…`→`f1d6b091…`): R1 nacimiento con el desborde declarado en
cabecera; R2 SB0 interno de cadena corta (sin pendientes: `CYC_FIN =
CYC_FROZEN + FROZEN_DEPTH`) **más la subida 512→1024 etiquetada como
el único cambio no-idéntico** — y el barrido de la regla §143.2 cazó
dos `512` de comentario más un «39 columnas» que YA venía desfasado
del legacy (deriva preexistente, documentada en el propio comentario);
R3 mundo envuelto (`COL_LEAF_SALT = 42`, sin colisión: burn no tenía
salt previo); R4 las seis (cola en `C_FBIT_BOOL`, el arnés de vacuidad
cubriendo las 24 en la misma pasada); R5+R6 en tanda — las dos
mutaciones de la spec §4 RECHAZAN, espejo nativo↔circuito con la resta
del carril B, frozen-32 local con `frozen_climb_32`.

**Estado**: tres gemelos de diez (send §143, claim §144, burn §145);
capa 240/2 intacta toda la campaña. La tabla §3 tiene su primera fila
con historia. SIGUE: **mint** — un tramo, sin fase frozen (R6 no
aplica) y la variante que la spec §5 advierte: sin carril de resta, a
censar en R0.

## 146. Mint COMPLETO — el primer gemelo sin frozen, y el titular asciende al mundo ancho

**Lo que el censo R0 desveló** (y que ningún calco de claim habría
visto): mint no es la máquina de send con menos fases — es **emisión
con autoridad de umbral 2-de-N**, DOS árboles (cuentas + custodios,
`CUSTODIAN_DEPTH` de `circuit_threshold`), columnas gemelas por
custodio, y **un solo llamador** para sus 13 tests. La máquina de hoja
sí es la estándar («sin carril de resta» = el carril B SUMA, como
claim), sin fase frozen (checklist: mint | salt sí | frozen-32 —) →
**R6 no aplica** y el rito se encoge a cinco eslabones.

**Los cinco commits** (`892874f`→`502e014`→`79e8576`→`198e5a9` con la
tanda R4+R5 fusionada; md5 `3129d036…`→`33b8cbcf…`→`5fd246df…`→
`3ca5777c…`): R1 nacimiento (guardián 30→31); R2 SB0 de **doble
árbol** — cadena `CYC_ACCT_ROOT`/`CYC_CUST`/`CYC_FIN = CYC_CUST +
CUSTODIAN_DEPTH`, identidad de las cinco `ROW_*` verificada
pre-escritura, y un patrón de frontera PROPIO documentado: el tramo de
custodios arranca **sin sombra** (su brazo previo es `ROW_ACCT_ROOT`,
que siembra la derivación de identidades, no el nivel 0 — el rango
coloca el nivel 0 él mismo); R3 mundo envuelto con la variante
conceptual del circuito: mint acredita a un TITULAR ajeno, y su salt
es del titular (§117) — el escenario **asciende al mundo ancho**: la
clave de juguete estrecha `0xA11CE` (pre-§90) se vuelve clave ancha de
verdad (§90.3), `derive_public_id_wide` + `derive_leaf_salt_wide` de
la MISMA clave, y `derive_public_id` estrecho muere del fichero;
R4+R5 en tanda — `P_LINK_SALT` entre `link_leaf` y `cust_link`, las
seis con su bloque de grados tras los enlaces de segmento, las dos
mutaciones RECHAZAN, espejo limbo a limbo con el carril B sumando.

**Presupuesto** (cuarta fila de la tabla §3, la primera sin columna
frozen): 311 → 319; holgura **192** (24 ciclos) en su 512. CABE.

**Nota operativa para el flip**: quien construya la prueba de mint en
el mundo nuevo necesita el `leaf_salt` del titular acreditado como
testigo — el dueño lo deriva de su clave; el emisor lo recibe del
dueño. La capa lo cablea en el flip (D4), aquí queda anotado.

**Estado**: cuatro gemelos de diez; capa 240/2 intacta. SIGUE:
**audit** («no muta estado», spec §5 — a censar qué significa para
los dos carriles y el espejo).

## 147. Audit COMPLETO — la variante de UN carril, y el molde demostró que se adapta

**Lo que el censo desveló**: audit no es un gemelo de menos fases —
es de **arquitectura distinta**: UN solo carril (ancho 27, un estado
Rescue), sin `COL_BAL_NEW` ni importes, con `COL_LOWER`/`COL_UPPER` —
prueba `saldo ∈ [inferior, superior]` **sin mutar estado** (la
advertencia de la spec §5, descifrada). Y la elegancia estructural que
lo hizo el gemelo de menos fricción: audit ya tenía **struct de
testigo** (`AuditWitness`) — el salt entró como campo, con **cero
firmas tocadas y cero llamadores enhebrados**.

**Las tres adaptaciones del molde** (primeras reales de la campaña,
todas al playbook): (1) R4 son **TRES familias, 12 ranuras**
(`C_SALT_{CAP,DIG,IN}` sin sufijos A/B, el estilo propio de audit);
(2) el espejo nativo↔circuito es de UNA hoja; (3) el hogar del salt es
el testigo, no la firma. El guardián de ranuras parseó la variante y
la bendijo sin tocar su código: el censo por `listdir` + convención
paga otra vez.

**La costura que el conteo no podía ver**: el test «NADIE PUEDE
REVELAR POR OTRO» construye su propio `AuditWitness` (el atacante),
con formato distinto al del escenario — el grep de literales del
struct lo cazó ANTES de compilar. Semántica registrada: el atacante
hereda `victim.leaf_salt` porque **el salt es observable; el secreto
es la clave** (§117) — el ataque sigue apuntando a titularidad
(`C_PK_CHECK`), no a pertenencia. Regla nueva del playbook R0: censar
TODOS los literales del struct-testigo, no solo el nexo.

**Y un tropiezo del asistente, cazado por su propia guarda**: el
ancla del marcador de sección se escribió de memoria («===== Filas
=====») cuando audit dice «===== Filas de eventos =====» — el conteo
esperado abortó con CERO bytes escritos, se leyó el natural, se
corrigió. La regla «anclas del natural, nunca de memoria», reafirmada
por la vía práctica.

**Los cinco commits** (`972dc25`→`1e7fbf9`→`36de0c9`→`20aa611`, con
R4+R5 en tanda; md5 `89cc95cb…`→`8ca55df1…`→`870a08c0…`→`e49e42ee…`).
**Presupuesto** (quinta fila): 279 → 287; holgura **224** (28 ciclos)
en su 512. CABE.

**Estado**: **CINCO GEMELOS DE DIEZ — mitad de campaña**; capa 240/2
intacta. SIGUE: mint_climb (un tramo).

## 148. Mint_climb COMPLETO — el gemelo exprés, y las derivas del legado documentadas

**Qué es**: la fase de cuentas de la emisión, aislada — hoja + subida,
dos carriles (el saldo SUBE), sin titularidad, sin custodios, sin
frozen. Nació de AMPUTAR mint (su propia cadena P lo cuenta: «Se
fueron P_CUST_LINK, P_POW2 y P_SEL_CUST_ROOT con la amputación»), y
esa historia dejó dos derivas que el censo cazó y este paso reparó:
(1) el doc de `ROW_ACCT_ROOT` hablaba de «arrancar la fase de
custodios» que aquí no existe — reparado con arqueología; (2) la
cadena P cuelga de `P_ACCT_LINK` y `P_FIRST_ROW = P_LINK_LEAF + 1` —
`P_LINK_SALT` entró re-enganchando al sucesor, respetando el hueco
histórico.

**Los cinco commits** (`f006c32`→`7e71f95`→`cfb95e8`→`04fd772`, R4+R5
en tanda; md5 `e83c6433…`→`6d177a65…`→`77c3479b…`→`74f151c9…`): R1;
R2 mínimo (tres `ROW_*`, `CYC_FIN = CYC_ACC + TREE_DEPTH`); R3 con el
ascenso al mundo ancho calcado de mint y `COL_LEAF_SALT` en **el
estilo derivado que la casa ya usaba** (`COL_SACC + 1`, ancho
`COL_LEAF_SALT + 4`); R4+R5 con el embudo `build()` de la casa
haciendo las mutaciones de una línea.

**Dos abortos de guarda en el camino, cero bytes escritos**: el
retorno compacto del escenario (una línea, no el par vertical) y el
`NUM_CONSTRAINTS` **pub** + cola de vacuidad compacta — ambas
divergencias de formato cazadas por el conteo esperado, corregidas del
natural. El sistema de anclas pagándose en el circuito siete.

**Presupuesto** (sexta fila): 271 → 279; holgura **232** (29 ciclos)
en su 512. CABE.

**Estado**: **seis de diez**; capa 240/2 intacta. SIGUE: **recovery**
— la variante anunciada más delicada de la spec: el salt se **COPIA**
(§93.4, costura 52), no se deriva fresco; su escenario refleja copia.

## 149. Recovery COMPLETO — LA COPIA (§93.4) hecha test

**La variante semántica de la campaña, resuelta.** Recovery reasigna
una cuenta (dos custodios, el precio dicho sin adornos en su propio
header): la identidad cambia (`id_old`→`id_new`), el nonce SALTA
(`+1`, invalidando lo firmado con la clave comprometida — su
restricción propia: `C_NONCE+1` ata contra `current[COL_NONCE] +
E::ONE`), el contador de recuperaciones sube… **y el salt NO cambia**:
el récord recuperado viste el de la clave VIEJA (§93.4, costura 52).
Estructuralmente eso es el gemelo estándar — UNA `COL_LEAF_SALT` para
ambos carriles — porque la copia es semántica, y vive donde debe: en
el escenario (`leaf_salt = derive_leaf_salt_wide(key_old)` con su ⚠️),
en el brazo, en la doc de las seis, y sobre todo en **el espejo**, que
certifica los ocho limbos: hoja vieja y récord nuevo, envueltos con EL
MISMO salt.

**El doble ascenso** (§90.3): las dos claves de juguete estrechas
(`0xA11CE`, `0xBEEF_CAFE`) se vuelven anchas de verdad; identidad Y
salt derivan de la clave — la vieja manda sobre el salt, la nueva solo
sobre la identidad. La rotación del salt queda donde la spec la dejó:
costura 52, fuera de alcance.

**Los cinco commits** (`9d22008`→`e82410b`→`4e10e3f`→`7de56e3`, R4+R5
en tanda; md5 `8a39cd6e…`→`26c5e4ba…`→`5c187f19…`→`51025f17…`). En R1
la guarda de árbol-sucio absorbió un doble-pegado accidental con CERO
bytes escritos — tercera vez que el rito se paga solo en el día.

**Presupuesto** (séptima fila): 311 → 319; holgura **192** (24 ciclos)
en su 512. CABE.

**Estado**: **siete de diez**; capa 240/2 intacta. SIGUE:
recovery_climb (un tramo), y detrás el par final SIN salt —
freeze/frozen_climb, gemelos mínimos de profundidad (R1+R2+R6 con
mutaciones propias).

## 150. Recovery_climb COMPLETO — el octavo, con la receta ya madura

**Qué es**: la fase de cuentas de la recuperación, aislada — la forma
de mint_climb con la semántica de recovery: dos carriles donde la
identidad cambia, el nonce SALTA (`C_NONCE+1` contra `current + ONE`)
y **LA COPIA** (§93.4) viste ambos carriles con el salt de la clave
vieja. Sin custodios ni frozen. Su raíz es **pub** —
`pub const ROW_ACCT_ROOT` — con la joya de doc que el legacy ya traía:
el relleno tras la 271 sale GRATIS aquí (no es fila de enlace), «a
diferencia de `circuit_frozen_climb`, §60.2» — anticipo exacto del
gemelo que viene.

**Los cinco commits** (`92180eb`→`942bd86`→`4040d32`→`e866d25`, R4+R5
en tanda; md5 `aebf382a…`→`e5355344…`→`774f0e41…`→`7260a046…`): R2 a
la primera con las anclas de mint_climb; R3 con UN aborto limpio de
guarda — recovery_climb escribe compacto (hoja nueva en una línea,
retorno en una, embudo en dos) lo que recovery escribía vertical —
tres anclas corregidas del natural, cero bytes mal escritos; R4+R5 a
la primera con el ancla del nonce-que-salta ya validada en §149.

**Presupuesto** (octava fila): 271 → 279; holgura **232** (29 ciclos)
en su 512. CABE.

**Estado**: **OCHO de diez**; capa 240/2 intacta. SIGUE: el par final
SIN salt — **freeze** y **frozen_climb** (checklist: «solo
profundidad»). El rito muda por última vez: R1+R2+**R6** (FROZEN_DEPTH
= 32 local reemplazando el import, §128) + mutaciones de profundidad
propias (un camino de 24 declarado como 32 debe RECHAZAR). Y
frozen_climb trae el relleno CARO que su primo acaba de señalar
(§60.2).

## 151. Freeze COMPLETO — la CASA de FROZEN_DEPTH gira a 32, y el rito mínimo estrena forma

**El hallazgo que reorganizó el tramo final**: freeze no importa
`FROZEN_DEPTH` — **lo define** (`pub const`, su línea 61 del legado);
el resto del sistema lo importa de aquí. Los gemelos de liquidación se
independizaron con frozen-32 local (send/claim/burn, §143-§145)
precisamente para no depender de esta casa; el gemelo de freeze es
donde la constante GIRA de verdad.

**El rito mínimo (R1+R2+R6), y por qué R2 importa aquí**: el cuerpo de
`build_trace` ya era genérico en profundidad (`next_cycle 
FROZEN_DEPTH`, `== FROZEN_DEPTH`, `− FROZEN_DEPTH − 1`, y hasta el
enlace de custodios derivado) — solo los tres `ROW_*` eran literales
(191/192/231) con docs en prosa de 24 («ciclos 0-23»). R2 los derivó
(`CYC_CUST = FROZEN_DEPTH + 1`, `CYC_FIN = CYC_CUST +
CUSTODIAN_DEPTH`) y reescribió los docs en términos derivados **para
que siguieran siendo verdad tras el giro**. Resultado: **R6 = una
línea** — `24 → 32` — y el calendario entero (raíz 191→255, custodios
→295, holgura 280→216 en 512) se re-derivó solo; el guardián de
ranuras lo bendijo sin inmutarse y los OCHO tests del legado corren a
32 vía `frozen_path()` derivado, intactos.

**Las mutaciones que estrenan categoría** (ningún gemelo con salt las
necesitó): (1) un camino de 24 niveles declarado ante el circuito de
32 **muere en la construcción** — el `debug_assert` de `place` sembrado
en R2 (o el índice fuera de rango en release) lo caza antes de que
exista traza; (2) la raíz vacía de 24 ≠ la raíz vacía de 32 — verdad
nativa que fija la incompatibilidad de mundos (§128).

**Sin salt por diseño**: el árbol de congelados indexa por número de
cuenta, sin hojas de cuenta — R3/R4/R5 no aplican (checklist: «solo
profundidad»).

**Tres abortos de guarda en el camino, cero bytes**: dos clausuras con
la misma sangría (resuelto anclando por nombre completo — regla nueva:
ante clausuras homónimas en forma, el ancla incluye el `let nombre =`)
y la cola de vacuidad multilínea. **Los tres commits**
(`4090d23`→`8f5bcab`→`0fb407f`; md5 `5ea4ab06…`→`62dee025…`→
`a4a35c7c…`).

**Presupuesto** (novena fila, la primera del régimen de profundidad):
231 → **295** (a 32); holgura **216** (27 ciclos) en su 512. CABE.

**Estado**: **NUEVE de diez**; capa 240/2 intacta. SIGUE: el último —
**frozen_climb**, con su 24 clavado en la línea 166 (§143.1) y el
relleno CARO que recovery_climb señaló (§60.2: allí la última fila SÍ
es de enlace).

## 152. Frozen_climb COMPLETO — el décimo: ajuste exacto, y la campaña del paso 3 CERRADA

**Los dos avisos del paso 2, resueltos por lectura**: el «24 clavado
en la línea 166» (§143.1) resultó ser PROSA del mundo viejo (docs del
módulo y del TRACE) e ÍNDICES de restricción (`result[24 + i]` es la
ranura 24..27, no profundidad) — el código profundo ya derivaba todo
de `FROZEN_DEPTH` importado, con `ROW_ROOT` incluido. Y el relleno
CARO (§60.2) no hubo que pagarlo: **se disolvió por geometría** — a
32, `ROW_ROOT = 255 = TRACE_LENGTH − 1`, el ajuste es EXACTO y no
queda fila que rellenar. Guarda en compilación:
`assert!(ROW_ROOT == TRACE_LENGTH - 1)`.

**El giro (R2+R6 fusionados — nada que derivar: ya venía derivado)**:
el import de `circuit_freeze` se vuelve `const FROZEN_DEPTH: usize =
32;` LOCAL (los gemelos no importan de otros gemelos; en el flip la
CASA §151 exporta 32 y el import vuelve). La prosa del mundo viejo,
**redimida**: el primer renglón del doc del TRACE («32 niveles × 8
filas = 256») era herencia de `dual_climb` que POR FIN es verdad
aquí; el párrafo de deuda del módulo pierde su tensión — ambos árboles
viven ya a 32 y así queda escrito. El contrato de la firma («llega el
camino de 32 del árbol de cuentas; se usan los FROZEN_DEPTH
primeros») hizo el resto: los llamadores no cambiaron y el camino
corto de 24 muere en el `assert!` de entrada — primera mutación. La
segunda: raíz vacía de 24 ≠ raíz vacía de 32.

**Los tres commits** (`5f799b0`→`1dc4249`, R2+R6 en uno; md5
`13be019c…`→`e21c69a9…`). Todo a la primera: cinco anclas, cero
abortos.

**Presupuesto** (décima fila, la única exacta): — → **255** (a 32);
holgura **0** en su 256. **CABE — exacto.**

**Estado: DIEZ GEMELOS DE DIEZ — LA CAMPAÑA DEL PASO 3, COMPLETA.**
Suite 420/0; capa 240/2 intacta; guardianes 35·37. Sigue: asiento de
cierre de campaña e informe de traspaso actualizado.

## 153. CIERRE DE CAMPAÑA — los diez gemelos del paso 3, en una página

**Lo hecho** (§143-§152, 47 commits): DIEZ gemelos `*_salted` nacidos,
derivados, envueltos (los ocho con salt) o girados a 32 (los dos de
profundidad), cada uno con sus mutaciones obligatorias rechazando y su
espejo nativo↔circuito verde. Suite stark: **420/0** (de 341 al abrir
el paso 3); capa **240/2 intacta** — ni un test de zk-ssl tocado en
toda la campaña; guardianes **35·37** (citas · circuitos censados).

**Los tres hallazgos estructurales**: (1) **burn NO cabe en 512** —
único desborde; TRACE propia 1024, coste ~2× crudo, la medición
apareada del paso 5 (§130) lo cuantificará; (2) **audit es de UN
carril** — tres familias/12 ranuras, espejo de una hoja, salt en el
testigo: el molde demostró que se adapta; (3) **freeze es la CASA de
`FROZEN_DEPTH`** y frozen_climb, a 32, ajusta EXACTO — §60.2 disuelto
por geometría, no pagado.

**Las variantes semánticas registradas**: el titular de mint asciende
al mundo ancho y su salt viaja del dueño al emisor (nota para el flip,
§146); LA COPIA de recovery (§93.4) certificada por espejo — mismo
salt, identidades y nonces distintos — y heredada por recovery_climb.

**Las reglas que el playbook ganó por la vía práctica**: anclas DEL
NATURAL, nunca de memoria (reafirmada por ~ocho abortos limpios de
guarda — CERO bytes mal escritos en toda la campaña); censar TODOS los
literales del struct-testigo, no solo el nexo; ante clausuras
homónimas en forma, el ancla lleva el `let nombre =` completo; los
comentarios se revisan EN el paso que mueve la geometría; y el patrón
R2-deriva-R6-gira para los gemelos de profundidad (freeze: R6 = una
línea).

**Lo que la campaña deja armado**: paso 4 (T2b-circuito), paso 5
(medición apareada §130, con el 2× de burn como dato esperado), y el
flip D4 con sus avisos anotados donde muerden. El informe de traspaso
actualizado acompaña este asiento como documento.

## 154. Paso 4 COMPLETO — T2b-circuito: la recuperación de §117, extremo a extremo por el AIR

**La promesa del legado, cumplida donde estaba escrita.** El módulo
`t2b_recuperacion_nativa` (entrada 50) dejó dicho que T2b-circuito «NO
se escribe hoy porque exigiría un AIR que aún no existe» — condición
que el paso 3 cumplió. Hoy los dos hermanos existen, lado a lado:

**El test positivo** (`t2b_circuito_solo_la_clave_produce_prueba_que_verifica`,
en el gemelo del piloto): el titular pierde TODO menos la clave;
rederiva identidad y salt (§117) — dos asertos lo certifican contra el
escenario —, toma de lo PÚBLICO camino y raíces, y `circuit_send_salted`
le firma una prueba que **VERIFICA**. La recuperación no es papel: es
un STARK aceptado.

**El reverso** (`t2b_circuito_diccionario_sin_salt_no_verifica`): el
atacante del diccionario, ahora contra el VERIFICADOR — con salt=0 su
testigo no reproduce la hoja del árbol y ninguna prueba verifica
contra las raíces públicas. El cegado de §117 muerde a nivel de
circuito, no solo de hash.

**La línea obsoleta del nativo, actualizada**: su cabecera apunta ya a
los hermanos de circuito, y deja anotada la mitad que falta por
diseño: **la versión por `apply` de la capa llega con el flip (D4)** —
la capa no habla salted antes por mandato de la estrategia C.

**Un commit, dos ficheros** (md5 send `e239fc5b…`→`1b9cfff5…`;
settlement `86b9ac26…`→`f6a5e552…`); un aborto de guarda por la cola
real del gemelo (SB1.d dejó las mutaciones al final), cero bytes.
Suite **422/0**; capa 240/2; guardianes 35·37.

**Estado**: paso 4 CERRADO. La cláusula de caída de §116 queda
decidida por partida doble — nativo Y circuito. SIGUE: paso 5, la
medición apareada (§130), con el 2× de burn como dato esperado.

## 155. Paso 5 COMPLETO — la medición §130 apareada: burn ×2,04 clavado, frozen_climb gratis, audit encoge

**El arnés**: veinte instrumentos `#[ignore = "instrumento de medida…"]`
(uno por circuito y lado, junto a su propio escenario — el patrón de
`metrics_33` y de los dos ignored históricos de settlement),
construcción + prove dentro del reloj, tamaño de prueba incluido.
Piloto send primero (validado con sus dos líneas), luego los nueve
pares en un mega-parche de 18 ficheros — **las dieciocho anclas a la
primera**, incluidas las colas pre-compactación de claim/burn. La
primera corrida (paralela) enseñó la trampa: contención cruzada en los
tiempos; la canónica es **en serie** (`--test-threads=1`), 1,06 s de
pared para los veinte proves. Tabla completa en
`doc/medicion-130-apareada.md`; instrumentos en `6ee02d7`
(+ piloto `dd03c23`).

**Los tres titulares**: (1) **burn ×2,04** (44,7→91,3 ms) — la
predicción de §145 al montar la TRACE 1024, clavada al decimal;
(2) **frozen_climb GRATIS o mejor** (−5,3 % tiempo, −1,4 % prueba) —
el legado ya pagaba 32 subidas (24 reales + 8 de relleno CARO, §60.2)
y el gemelo cobra las 32 de verdad por el mismo precio: la capacidad
×256 sale regalada por geometría, cerrando el círculo de §152;
(3) **audit encoge** — prueba deterministamente menor (−653 bytes) con
carril único y salt; hecho anotado sin teoría forzada.

**El coste del mundo nuevo, medido**: fuera de burn (resuelto y
cuantificado), el salt + frozen-32 cuesta **0–9,5 %** de tiempo y
−2,2…+8 % de prueba por circuito — freeze paga sus 8 ciclos reales de
más (+9,5 %) y los demás viven en el ruido o cerca. El flip D4 tiene
ya su precio en la mano.

**Estado**: paso 5 CERRADO — los cinco pasos de B13/B14 previos al
flip, COMPLETOS (2: piloto §143 · 3: diez gemelos §144-§153 · 4:
T2b-circuito §154 · 5: esta medición). SIGUE: **el flip D4** (release
única: capa + gemelos + migración; los legacy se borran; snapshot v7;
avisos de §143.1/§146/§149-§152 cableados; la etiqueta de la entrada
50 se cierra ahí).

## 156. FLIP D4 — LA RELEASE ÚNICA: el mundo nuevo es EL mundo

**Lo que el commit `2ac37c5` hace** (35 ficheros, +2.543/−15.078): los
DIEZ gemelos SUSTITUYEN a sus legacy —que mueren—, `FROZEN_DEPTH` vive
a 32 en la CASA y send/claim/burn/frozen_climb vuelven a importarla
(helpers `_32` y consts locales, fuera); la capa habla salted de punta
a punta —`AccountView` porta el salt del record y viaja
materials→trace; 8 call-sites + barrido de construcción con
`leaf_salt_rec`/`LEAF_SALT_LEGACY`; LA COPIA en recovery (§93.4)—;
snapshot **v7** (hoja envuelta, frozen-32) con importación ≤v6 pineada
a `FROZEN_DEPTH_PRE = 24`; marcador `meta:geometry_v7` al persistir,
separado del acta `meta:migrated`; y el **museo** `circuit_settlement`
queda en 24 local con clímber propio hasta su retirada. Guardián de
ranuras: **27 circuitos** —el censo del mundo nuevo, por primera vez—;
suites **316/0 + 12** y **240/2 + 3**.

**Los tres descubrimientos de la batalla.** (1) El museo: un undécimo
circuito que ningún censo contó porque no está vivo — trece tests
suyos cayeron dos veces (geometría corrida, luego el clímber de la
CASA a 32 sobre sus caminos de 24) y ambas el pánico señaló la línea
exacta. (2) El bug REAL de rotación: derivar el salt de la clave en el
cliente rompe LA COPIA tras recovery —`derive(clave_nueva) ≠ salt del
record (clave vieja)`—; el salt no se deriva en el cliente: **viaja en
los materials**, y esa es la razón de ser del campo en `AccountView`.
(3) El marcador dividido: «persistí en geometría nueva» y «pasé por la
migración» son afirmaciones distintas — el guard anti-re-ejecución
exigía la separación y un test la hizo valer.

**El método, bajo fuego real.** El espejo del asistente DERIVÓ entre
turnos (el entorno se re-aprovisionó con su propio núcleo aplicado en
una cola de turno perdida) y de la confusión salió doctrina: **el
canon es el árbol del piloto** — los md5 del espejo pasan a
informativos y los jueces son los asertos de cada transformación, los
guardianes y las suites. Siete rondas de A3 con ~cinco abortos limpios
de guarda (la segunda llamada `frozen_climb_32` de burn que el censo
dejó anotada como duda, el estado-desconocido que enseñó a detectar
«hecho» por estructura, y el E0062 que RESOLVIÓ el fantasma: los
literales ya llevaban el salt más abajo de mis ventanas de lectura).
**Cero reversiones; cero bytes escritos por una guarda rota.**

**Cola.** **F3**: migración en vivo sobre los tests de capa —los DOS
rojos históricos de privacidad de índices mueren con la reposición
`public_id mod cap`— y la etiqueta de la entrada 50. **F4**: flecos
censados —doc de `client.rs:490`, expectativa del instrumento-barrido
(ahora §117 manda que NO encuentre), prosa «GEMELO» en el doc de
medición, epílogo del playbook, retirada futura del museo, warnings—.
El mundo viejo ya solo existe como historia bien contada.

## 157. F3 COMPLETO — los contratos cobran, CERO ROJOS, y la entrada 50 se etiqueta

**La colocación del mundo nuevo** (`e3116c6`): `open_account` adopta
la política de la migración — `public_id[0] mod capacidad` con sondeo
lineal — y `next_index` queda como CENSO (la cuota), no como posición.
Los DOS contratos históricos de privacidad
(`account_indices_are_not_predictable` / `_not_enumerable`) cobran: un
atacante ya no elige vecino con sus altas ni enumera cuentas barriendo
índices. La migración queda como herramienta de stores LEGACY — sobre
mundo fresco es IDENTIDAD, y su test de congelación ahora lo documenta
migrando desde un legacy SIMULADO (récord plantado a mano en el 0).

**Los cinco fósiles de la era secuencial** (`f926de6`): `capa()` de
los tests delegada devolvía `(l, 0)` en vez del índice real (ocho
tests); el test de congelación presumía viejo≠nuevo; `balance_of(0)`
cableado tras el reinicio cifrado; `let alice = 0` en el ataque del
tope; y el invariante de self-send sumaba sobre `0..censo` en vez de
sobre los RECORDS. Cinco cortes con ancla, y el número que la casa
esperaba desde su fundación: **zk-ssl 242/0 + 3 · stark 316/0 + 12 —
CERO ROJOS en ambas suites por primera vez.**

**El traspié, al registro**: el Bloque B de F3 llevaba condición
«solo con doble verde» y disparó sobre 12 rojos — `e3116c6` quedó
empujado afirmando un 242/0 que aún no existía. Doctrina fix-forward:
`f926de6` endereza el estado y lo dice en su propio mensaje. Regla
ganada: **el mensaje de un commit no afirma un verde futuro** — se
escribe tras el verde o se escribe condicional.

**El regalo de la campaña**: el instrumento-barrido de client
(`#[ignore]`) ya afirmaba `hallado.is_none()` — era un contrato
durmiente que el mundo salted puso en verde SOLO. Un fleco de F4,
disuelto antes de nacer.

**Etiqueta**: `entrada-50` anotada sobre `f926de6`. La entrada más
larga del BACKLOG — salt de hoja (§117), frozen-32 (§128), diez
gemelos, flip D4, colocación — queda CERRADA de punta a punta. Cola:
solo **F4** (flecos censados: doc de `client.rs:490`, prosa «GEMELO»
en el doc de medición, epílogo del playbook, retirada futura del
museo, warnings).

## 158. F4 COMPLETO — CERO·CERO, la retirada del museo registrada, y la COLA VACÍA

**Los flecos, en cuatro cortes** (`a2f414d`→`af75c0a`): la prosa del
mundo viejo pasó a pasado y el contrato del barrido habla en presente
(§117); el doc de medición ganó su postscriptum y el playbook su
epílogo con las reglas de fuego; los módulos `t2b`/`t2a` recibieron su
puerta `#[cfg(test)]` — **compilaban en la lib sin ella**, un latente
que el foco del flip sacó a la luz —, el clímber del museo quedó
enjaulado y la voz de `GASTO` protegida; y los dos `deprecated`
deliberados llevan `allow` con puntero. Un disparo de B sobre A
abortado salió VACUO («nothing to commit»): la atomicidad de la cadena
pagó una vez más.

**Las dos voces que quedan anotadas, y de quién son**: el shim de
`record_to_bytes` pertenece a **49-A** (su test lo ejercita a
propósito) y el `mint` de operador a la **entrada 32** (inventariado
en AUDITORIA 80). No son deuda de B13/B14: viven registradas en sus
atributos y en sus asientos, y migran con sus dueños.

**La retirada del museo, REGISTRADA como futura** (sin fecha):
`circuit_settlement` — trece tests, frozen-24 local, clímber propio.
⚠️ Condición de retirada: **los helpers nativos se mudan ANTES**
(`native_leaf`/`native_leaf_salted`/`derive_*` viven en ese fichero y
son del mundo vivo — el museo se va, la fragua se queda).

**El número final**: warnings **0 · 0** y suites **316/0 + 12 ·
242/0 + 3**. CERO·CERO por primera vez.

**COLA DE B13/B14: VACÍA.** La entrada 50 —salt de hoja (§117),
frozen-32 (§128), diez gemelos, T2b doble, medición, flip D4,
colocación pid-mod, flecos— queda cerrada de punta a punta, etiquetada
(`entrada-50`) y contada donde sobrevive (§140-§158). Lo que el
proyecto aún debe pertenece a otras entradas, y cada una lo sabe.

## 159. Campaña A COMPLETA — el triaje de las 14, reconciliado con la jornada, y las tres plumas

**El triaje (2026-08-03, doc/triaje-14-medidas.md; citable en la
entrada 70) reconciliado con el día del flip**: la **3b** pasó de
*diseñada* a **HECHA** (entrada 50, etiqueta); la **3a** sigue vetada
por §129 y hoy además existe la vista autenticada; la **4a** sigue
siendo la 32; los preprints siguen intocables por §135 (errata) — pero
los **docs VIVOS** habían quedado mintiendo y la 14-viva se ejecutó:
seis censados, dos con desactualización real.

**A-1 (`98f0813`)**: README y SECURITY pasan las dos superficies
medidas del 31-07 a **RESUELTO** — la tabla y la curva del diccionario
se conservan como historia del riesgo cerrado; la objeción de §99.3
(«escribir sin conocer el secreto») queda disuelta por el
salt-en-récord con LA COPIA; la enumerabilidad y la elegibilidad del
vecino, muertas por la colocación pid-mod con sus contratos en verde.
La spec de migración, sellada **EJECUTADA**.

**A-2 (`567eed1`) — las tres plumas (medidas 7/12/13)**:
PRINCIPIOS §6.bis fija la relación con el «último intermediario» y el
dinero cuántico en tres registros honestos (eliminado / reducido con
ataque diseñado —el acuse §121— / fuera del dominio clásico); VISION
apunta; SECURITY §2.bis nombra los DOS residuos que quedan (orden y
completitud; el operador ve el estado) y qué los elimina; y
`doc/POSICIONAMIENTO.md` nace como la página pública — qué es, qué lo
distingue, qué NO es todavía, y la conversación que se propone
(piloto de pruebas de banda tras auditoría externa).

**Del triaje quedan vivas**: la **9** («qué aprende cada
participante», la mejor — Campaña C), la **6** (notas/UTXO, sesión
propia), la **10** (interfaz de anclaje, conecta con el acuse), y el
bloque **1/2/4a/11 = entradas 32/33 = Campaña B**, hoy reencuadrada
por censo: la vía delegada EXISTE completa
(`apply_*_delegated` ×4) y el orden estricto de §51 ya vive en los
appliers — el trabajo es migrar ~229 llamadas de la vía antigua,
amputar `transfer` y los `deprecated`, y medir después (11).

## 160. Campaña B ABIERTA — el censo canon reconcilia el ~229 del sello con el paisaje real

**El re-conteo (canon, doble verde con el espejo del sello
`63ca178`)**: la vía antigua directa —`mint`, `set_frozen`, `recover`,
`update_custodians`, `mint_to_pending`— suma **83 aciertos** (69 en
`tests.rs`, 3 en `tests_support.rs`, 11 repartidos entre definiciones
propias, usos inventariados —p. ej. accounts:298— y prosa técnica).
`open_and_fund` suma **185 usos** (tests 105, client 29, snapshot 19,
metrics 8, two_phase 8, migration 7, log 3, accounts 2, iso 2,
recovery 1): **la palanca de la campaña** — las DOS anchuras
(`open_and_fund` y `open_and_fund_wide`) montan sobre el `mint()`
viejo, así que migrar los helpers por dentro convierte 185 sitios en
uno. El ~229 del sello venía del espejo pre-flip; queda reconciliado.

**`transfer` está HUÉRFANO, no solo sin deprecar**: nada lo cablea
—ni `mod transfer` en lib.rs, ni `include!`, ni rastro en Cargo.toml—
y cero llamadores reales de `.transfer(`. Quedan 267 líneas muertas,
6 menciones-prosa (iso ×3, two_phase ×2, tests:3143 — esta habla en
PRESENTE de un test que ya no la usa) y un **fantasma doctrinal**: el
doc de transfer.rs:17 recomienda `client::prove_transfer`, que no
existe en el workspace (ni `transfer_materials`). La 32 pasa de
cirugía a retirada de cadáver + barrido — la entrada 70 ya lo olía:
«su retirada ES la 32 entera».

**Reconciliaciones con el sello**: la vía delegada son CINCO, no
cuatro — `apply_governance_delegated` también existe—; el molde de
fabricar `(subida, pa, ia, pb, ib)` vive DUPLICADO en el bloque de
tests de cada módulo (5-6 aciertos `_delegated(` por fichero) y
`tests_support` NO tiene fabricador común — esa extracción es la
primera piedra. Fósiles a barrer: el «145 veces» de tests_support:6
(hoy 185) y la prosa en presente de tests:3143.

**Alcance (ratificado con este asiento)**: `open_account` de 64
bits (accounts.rs:88) queda FUERA de B — su migración es opt-in por
§97.4 (§90 garantiza identidad idéntica con clave rellenada) y el
sello jubila solo la vía de custodios. El shim de store.rs:189
tampoco entra: migra con 49-A, como declara su propia nota.

**Las tandas**: **B-0** fabricador delegado común en tests_support
+ `open_and_fund{,_wide}_delegated` (la palanca) · **B-1** la 32:
retirar transfer.rs + barrido de prosa, fantasma y fósiles · **B-2**
las ~70 llamadas directas de tests.rs, por familias · **B-3** jubilar
las cinco `#[deprecated]` de custodios · **B-4** medir (medida 11):
`metrics_of_the_layer` + patrón §130; tablas vivas sí, preprints por
errata (§135). Preside el matiz de §51: el orden estricto ya vive en
los appliers — los tests migrados deben EJERCITARLO, no esquivarlo.

## 161. La 32-transfer EJECUTADA — el cadáver sale del árbol, y dos letreros dicen hoy la verdad

**Lo retirado**: `transfer.rs` entero (267 líneas — `transfer()` y su
`apply()`), el `struct Settlement` de lib.rs (huérfano con él) y
`SettlementPublicInputs` del import de la raíz, sin lector al caer el
struct. `native_leaf` NO era huérfano: un `head -4` truncó su censo
de lectores en el espejo y persistence:331 —el único que bebe del
import de la raíz— quedó fuera; el compilador lo devolvió y la
compuerta hizo su trabajo (cero empujado). Regla ganada: **los censos
de poda se cuentan SIN `head`**. Nada lo cableaba —ni `mod`,
ni `include!`, ni Cargo— y nadie lo llamaba: **el flip ya lo había
dejado fuera del árbol de compilación**; esta tanda entierra el
cuerpo. Con él muere el fantasma doctrinal de §160: el doc que
recomendaba `client::prove_transfer` se va con su fichero.

**Dos reconciliaciones de doctrina, ambas a favor de HOY**: (1) §32
preservaba `apply()` «porque el cliente la necesita» — el cliente por
compromisos (`prove_transfer`/`transfer_materials` sobre
`circuit_settlement`) murió en el flip, así que la fila «viva» del
cuadro de §32 es historia y `apply()` cae con el fichero: la
superficie del nullificador que §32 vigilaba queda cerrada por
retirada total (el árbol ya se había ido; véase la entrada 1). (2) El
letrero de two_phase («cuando `transfer()` desaparezca, esto
también») **prometía de más**: `records` no sirve a la verificación,
pero alimenta los accessors del operador (§129; `balance_of` lee
`records`) y el snapshot — el insert se queda y el letrero ahora lo
dice.

**El barrido de prosa**: cuatro presentes falsos pasan a pasado (iso
×2, two_phase, tests — la vía clásica «llamaba», el test viejo
«usaba») y el fósil «145 veces» de tests_support pasa a 185, el
número del censo canon (§160). El bloque histórico de iso
(«RETIRADO… Llamaba») ya hablaba en pasado y no se toca: los
comentarios pueden narrar historia; lo que no pueden es mentir en
presente.

**Verificación de coste cero**: suites y guardianes idénticos al
turno anterior — quitar lo que no compila no puede mover un dígito, y
que no lo mueva es exactamente la comprobación. La entrada 32 NO
viste ✅: le quedan el cronómetro de §80.5 (= B-0b), la migración de
llamadores (B-2) y la jubilación de las cinco marcas (B-3).

## 162. El precio de la palanca, MEDIDO — y el tenedor de §80.5 sobre la mesa

**Instrumento**: `el_precio_de_la_palanca` en metrics.rs, apareado en
el mismo proceso —fondear por la vía vieja (UNA prueba: mint) y por la
delegada (TRES: subida + par de umbral §51-estricto)— bajo protocolo
§89.1: una ejecución del proceso = una muestra, cinco muestras,
release, rango y no cifra (§131).

**El canon**: vieja **108,4–109,7 ms** (dispersión ~1 %); delegada
**388,2–403,1 ms** (~4 % — más piezas móviles, más varianza, en el
espíritu de §130.4); multiplicador **×3,54–×3,70**. La entrada 32
avisaba «tres pruebas donde la antigua genera una» y el número le da
la razón: el coste escala con el número de pruebas — la apuesta previa
de un multiplicador suave era errónea y queda retirada por medición.

**Proyección para la suite** (explícita en sus supuestos): 185 usos
censados de `open_and_fund{,_wide}` (§160) × ~0,28 s de sobrecoste ≈
**+52 s de techo** si cada uso ejecuta una vez por pasada — del orden
de 23 s al orden de 70-80 s. La cifra real la dirá el flip y se
publicará como rango; los 69 llamadores directos de tests.rs (B-2)
añadirían lo suyo con la misma aritmética.

**El tenedor de §80.5, ahora con el número en la mano**: (a)
**retirar** — el sobrecoste es de CI, no de usuario: segundos de
máquina por que la suite ejercite la vía REAL, que es cobertura y es
la tesis («que nadie pueda pedir las claves»); (b) **no retirar y
declararlo**, precedente §46 con la entrada 6, dejando las cinco
marcas y el porqué escrito. Recomendación del asistente: **(a)**,
porque el precio compra exactamente lo que la campaña promete y se
paga donde menos duele. **DECISIÓN: del piloto, al abrir B-0b-ii** —
este asiento registra el número; la palabra siguiente registra el
rumbo.

## 163. La decisión de §80.5, derivada de los principios — RETIRAR, y la llave girada

**El mandato del piloto**: decidir aplicando de forma coherente,
transparente y honesta los principios fundacionales, como imagen fiel
de la realidad del proyecto. La derivación, con los textos delante:
(1) el objetivo nombrado (§80.5): «el objetivo no es borrar cinco
funciones: es que nadie pueda pedir las claves»; (2) la garantía
(§65.5, §80.6): «la garantía… no la da esta marca, la da USAR la vía
delegada» — mientras existan, *evitable, no cerrado*; (3) la doctrina
de retirada ya sentada por esta casa en la 32-transfer: «una fuga
presente y alcanzable es una fuga, aunque exista una alternativa
mejor al lado. Por eso se retira en vez de marcarse como
desaconsejada»; (4) el método fundacional (PRINCIPIOS §1): «las
propiedades verificadas una a una y el coste de cada una medido» — el
coste está medido (§162) y se paga en máquina, no en usuario; (5) el
test del precedente §46/entrada 6: allí el código corrigió una
premisa y conservar quedó casi forzado; aquí ninguna premisa corrige
a favor de conservar — la delegada existe ×5 y la paridad está
probada (B-0a); (6) imagen fiel: una suite cuya mitad depende de la
vía marcada *sin nombrarla* no describe lo que el proyecto declara.
**Seis señales, una dirección: RETIRAR.** La rama (b) de §80.5 queda
descartada con sus razones a la vista.

**Lo ejecutado**: `open_and_fund` y `open_and_fund_wide` fondean por
`fund_delegated` — 185 usos pasan a ejercitar la vía real en cada
pasada. Los dos tests de paridad re-anclan su lado viejo a la vía
antigua EXPLÍCITA (`mint` + `apply_mint` a la vista, bajo el `allow`
del fichero) y siguen montando guardia: delegada == vieja en
identidad, saldo, nonce, suministro y colocación, hasta que B-3
jubile la vieja y ellos se adapten con ella. Los letreros de
tests_support cuentan hoy: el fondeo ya no es lo que el `allow`
ampara.

**El coste, pagado donde se dijo**: la suite en modo nuevo da su
primera muestra en el verde de este mismo bloque; el rango se
completará con las pasadas venideras (§131), contra el techo
proyectado en §162 (~+52 s). La entrada 32 viste el paso §80.5 como
EJECUTADO; le quedan B-2 y B-3 para su ✅ entero.

## 164. B-2 ABIERTA — el mapa del cupo completo, la doctrina de migración, y seis mecánicos girados

**El mapa del cupo, por función contenedora — con corrección**: la
media-respuesta del traspaso («mint en ninguna») venía de greppear el
término equivocado; el mapa real es PARIDAD PERFECTA —
`consume_custodian_use` vive en las dos vías de las CUATRO
operaciones que consumen: mint (apply_mint :125 / delegada :290),
freeze (:132/:243), recovery (:163/:306), mint_to_pending
(:786/:950) — y **governance no consume**: la rotación renueva el
cupo, no lo gasta (`rotating_the_custodian_set_renews_the_quota`).
Consecuencia doctrinal: **los quota-tests migran limpios** — la
delegada gasta exactamente donde gastaba la vieja.

**El censo fino de tests.rs, al registro**: 68 llamadas directas en
47 tests — mint 27 (+1 comentario), set_frozen 17, update_custodians
10, recover 9, mint_to_pending 5. Mecánicos por patrón generalizado
(`let R = capa.mint(...).expect(); capa.apply_mint(&R,...)`):
**nueve**, de los que **tres reusan el receipt** después
(`replaying_a_mint_is_rejected`, `an_exhausted_custodian_set…`,
`a_restart_does_not_renew…`) y quedan para su tanda semántica.
Doctrina de la campaña, estilo §77: cada test **migra** (setup por la
delegada), **se porta** (la conducta, por la vía nueva) o
**muere-con-la-marca en B-3 con su porqué escrito** — nada se pierde
en silencio.

**B-2a ejecutada**: los seis mecánicos limpios giran a
`fund_delegated` — `minting_exactly_to_the_cap_is_allowed`,
`burning_frees_up_minting_capacity`,
`each_custodian_intervention_consumes_quota`,
`rotating_the_custodian_set_renews_the_quota`,
`the_custodian_quota_survives_restart`,
`changing_custodians_revokes_the_old_ones`. Los cuatro quota-tests
pasan a CONTAR intervenciones de la vía real, con el mapa de arriba
como garantía de que cuentan lo mismo. Coste esperado: ~+1,7 s de
suite (6 × ~0,28 s, §162).

## 165. B-2b — los tres con-reuso, leídos a los ojos: dos portan, uno releva

**El detector confesó un falso positivo**: en `an_exhausted…` y
`a_restart_does_not_renew…` el «reuso» del recibo era *shadowing*
(`let m` re-ligado para el intento fallido) — más llamadas viejas del
mismo test, no reuso. Solo `replaying_a_mint_is_rejected` reusa de
verdad: reaplica el MISMO recibo, que es exactamente su propiedad.

**Dos portan enteros**: agotamiento del cupo y su persistencia tras
reinicio son propiedades del cupo —que vive en paridad :125/:290
(§164)— y ahora se ejercitan POR LA DELEGADA: el bucle consume con
`fund_delegated`, el intento que debe rebotar usa materiales crudos
(`mint_commitment` + `mint_climb_proof` + `delegated_pair`) contra
`apply_mint_delegated`, y rebota en `CustodianSetExhausted` — el
cupo se gasta DESPUÉS de verificar la autoridad, en las dos vías. El
letrero interno de restart se generaliza: «el cupo se consume en el
APPLY», sin nombrar vía.

**Uno releva**: la propiedad «nada se emite dos veces con los mismos
materiales» gana su guardián delegado —
`replaying_a_delegated_mint_is_rejected` en el bloque de mint,
materiales clonados y reaplicados sobre el estado ya mutado, rechazo
por is_err con saldo y suministro clavados—. El test viejo de la
vía-recibo QUEDA (guarda una función que aún existe) y **muere con la
marca en B-3, relevado**: primera entrada del libro de cobertura de
la campaña. Suite: 244 → 245.

**Y la compuerta destapó un flake latente, ajeno en contenido**: el
rojo fue `corrupted_ledger_is_detected_at_startup` — intacto por esta
tanda — cayendo en `sled::open` CRUDO al manipular el db: tras soltar
la capa, sled tarda en liberar el cerrojo (WouldBlock), y el rito ya
tenía `open_retry`… para la capa; la manipulación quedaba fuera. El
peso nuevo de la suite movió el reloj y la carrera asomó. Cura de
especie: `sled_open_retry` en tests_support (mismo rito, diez
reintentos) y el único sitio censado —SIN head— portado. Rojo ajeno,
gatillado en tiempo: exactamente para eso está la compuerta.

## 166. B-2c — la familia mint, CERRADA: el hallazgo del tope, dos portes, dos relevos y el libro

**El hallazgo que reclasifica**: la vía-recibo impone el tope EN LA
GENERACIÓN (`mint()` mismo devuelve `SupplyCapExceeded`); la delegada
lo impone EN EL APPLY por diseño — la generación es del cliente y no
conoce el suministro. Los cuatro tests gen-time (:267 simple, :293
acumulado, :322 lleno, :1231 tras reinicio) guardan un contrato de la
vía-recibo que muere con ella: quedan hasta B-3, relevados hoy en su
faceta-apply.

**Lo ejecutado**: `minting_increases_supply_exactly` porta entero —
materiales de cliente, la no-mutación del estado intacta («generar
materiales no muta»), la subida medida en bytes—; `burning_frees…`
completa su porte (el tope-lleno y el margen-recuperado, por la
delegada). Dos relevos nacen en el bloque delegado:
`the_delegated_cap_is_enforced_at_apply` (forma acumulada, que
subsume la simple; el rechazo llega ANTES de verificar el par) y
`materials_for_one_account_do_not_apply_to_another` (el compromiso
ata el índice). Suite: 245 → 247.

**El libro de la familia, con destino escrito**: viven-hasta-B-3 y
mueren relevados — :55 autoridad-mala (relevos existentes:
`governance_keys_cannot_mint`, `the_same_custodian_twice`); :267/
:293/:1231-1240 tope-en-generación (relevo de hoy, faceta-apply);
:679 replay (relevo §165); :730 recibo-a-cuenta-ajena (relevo de
hoy); :1077 generar-no-consume (por construcción: la generación
delegada es del cliente; queda guardando el contrato viejo); y el
clúster de rotación :1592/:2154-2167/:2322, cuyo relevo —materiales y
claves viejas contra raíz nueva, §54.2 en vivo— tiene casa natural en
la familia governance (B-2e). **Cero llamadas de mint sin destino.**
Siguiente: set_frozen (17).

## 167. B-2d — la familia freeze, CERRADA: quince mecánicos, un relevo, paridad del contador

**La familia más disciplinada del censo**: 15 de 17 eran el patrón
mecánico puro —todos setup de tests cuyo sujeto es otro— y giran en
barrido estructural (nombres asertados) a `set_frozen_delegated`,
sobre la palanca extendida: `freeze_commitment` (raíces del árbol de
congelados + `count → count+1` atado), `freeze_climb_proof`
(FrozenClimb) y el aplicador. Paridad del contador público verificada
del natural ANTES de girar: `freeze_count` avanza en las dos vías
(:142 vieja, :246 delegada) — `every_freeze_is_counted` cuenta hoy
intervenciones reales.

**Los dos no-mecánicos, al libro**: `replaying_a_freeze_is_rejected`
vive hasta B-3, relevado hoy por
`replaying_a_delegated_freeze_is_rejected` — los mismos materiales
reaplicados rebotan porque el contador ya avanzó, y el contador no
avanza dos veces (que es exactamente lo que el contador existe para
garantizar) —; `one_custodian_cannot_freeze_alone` (autoridad-mala en
generación, contrato de la vía-recibo) vive hasta B-3 con relevos
existentes: `the_same_custodian_twice_cannot_freeze` y
`governance_keys_cannot_freeze`. **Cero llamadas de freeze sin
destino.** Suite: 247 → 248. Siguiente: update_custodians (10), la
casa natural del relevo de rotación que mint dejó pendiente (§166).

## 168. B-2e — la familia governance, CERRADA, y la deuda de §166 SALDADA

**Paridad del contador de cambios verificada del natural** (:160
vieja, :258 delegada) y palanca extendida al dominio de gobernanza:
`governance_commitment` (saliente → entrante, contador atado),
`governance_pair` (miembros, §51-estricto) y
`update_custodians_delegated`. **Cinco mecánicos giran** en barrido
estructural — rotación-renueva, invalida-pendientes, revoca-viejos,
cada-cambio-contado, conjunto-vigente-sobrevive: la rotación de los
tests es hoy la rotación real. El censo confesó un séptimo aparente
que no lo era: `the_governance_set_survives_restart` es SONDA de
generación (`is_ok` sin apply) — contrato gen-time de la vía-recibo,
al libro.

**La deuda de §166, saldada**:
`rotation_orphans_pregenerated_delegated_materials` vive entre los
tests de gobernanza y ejercita §54.2 en vivo — materiales de emisión
generados bajo el conjunto vigente, rotación delegada, y ni los
materiales ni las claves del saliente sobreviven: la capa pone SU
raíz. Releva en B-3 al clúster vía-recibo (:1592, :2154-2167,
:2322).

**El libro de la familia**: `replaying_a_governance_change` vive
hasta B-3, relevado hoy — el reintento delegado choca en la puerta
`RecoveryToSameIdentity`, con el compromiso como segunda cerradura —;
la sonda gen de :1362 y el trío de autoridad-mala (:1393 impostor,
:2202 custodio-no-cambia, :2229 gobernador-solo) viven hasta B-3 con
relevos existentes: `the_same_governance_member_twice_is_rejected`,
`custodian_keys_cannot_change_the_custodian_set`,
`an_authorization_for_another_root_does_not_apply` — y la sonda se
releva en B-3 con una operación delegada post-reinicio. **Cero
llamadas de governance sin destino.** Suite: 248 → 250. Quedan:
recover (9) y mint_to_pending (5).

## 169. B-2f — la familia recovery, CERRADA: las dos puertas y el recibo que cruza el reinicio

**Hallazgo primero — la puerta que NO estaba**: la vía-recibo rechaza
recuperar-a-la-misma-identidad en la GENERACIÓN (:72)… y la delegada
lo ACEPTABA. El relevo nació afirmando «puerta en el circuito» y el
rojo lo desmintió con un `Ok(())` — ni capa ni circuito la tenían, y
el no-cambio gastaba cupo, avanzaba contador y quemaba nonce. **Cura
en esta misma tanda**: la puerta vive ahora en
`apply_recovery_delegated` (calco de la de gobernanza), rechaza ANTES
de verificar, y el relevo —reformado a `matches!` y sin-gasto— la
vigila. Primera divergencia real de guardas que la campaña destapa y
cierra, y la razón de ser de los relevos: un guardián que nace en
rojo vale más que diez que nacen en verde.

**Hallazgo segundo — el recibo que cruza el reinicio**: la guarda de
reuso destapó que `the_recovery_counter_survives_restart` no era
setup: `recibo = r` saca el recibo del bloque para REAPLICARLO tras
reabrir — es el guardián de réplica-tras-reinicio de la vía-recibo.
Al libro: mecanismo relevado hoy (el replay delegado ata raíces y
contador, que PERSISTEN), adaptación restart-con-materiales en
B-3.

**Lo ejecutado**: palanca con LA COPIA fiel (§93.4: salt y saldo se
preservan, el nonce avanza) — `recovery_commitment`,
`recovery_climb_proof` (nueve argumentos, calcada) y
`recover_delegated`; paridad del contador verificada (:182/:310);
**cinco mecánicos giran** (cuatro tests): no-levanta-congelación,
bloqueo de la clave comprometida, preservación de saldo y suministro,
contador ×2. **El libro**: replay y réplica-tras-reinicio relevados
hoy; `one_custodian_cannot_recover_alone` con relevos existentes
(mismo-custodio, otra-identidad, claves-de-gobernanza); same-identity
relevado por la puerta-circuito. **Cero llamadas de recover sin
destino.** Suite: 250 → 252. Queda: mint_to_pending (5).

## 170. B-2 CERRADA — la última familia, el libro consolidado, y la mesa puesta para B-3

**B-2g, pending**: palanca extendida (compromiso del árbol de
pendientes con suministro atado, subida de siete argumentos, y
`mint_to_pending_delegated`) y el quota-test girado — la emisión a
pendiente de los tests cuenta hoy intervenciones reales (:786/:950).
Al libro: el ciclo completo REUSA su recibo (vive hasta B-3; su
pierna-mint ya está relevada por el positivo delegado del módulo, y
el ciclo entero se adapta con el *aviso* en la jubilación); el
mismo-custodio gen-time con relevo en el bloque delegado; y el par
del tope en generación (`OverRegulatoryLimit`), contrato vía-recibo
cuya faceta-apply guarda la puerta `would_be` del apply delegado —
precedente del relevo-cap de mint (§166).

**El cierre de la campaña de migración, en números**: 68 llamadas
directas en 47 tests → **33 giradas** al barrido estructural (6+5
mint, 15 freeze, 5 governance, 5 recovery... y 1 pending) + 2 portes
enteros + **9 relevos nuevos** en los bloques delegados + ~20
entradas de libro, cada una con su relevo o su porqué. Suite 244 →
252, tiempo 19-27 s → 34-41 s: el precio de §162, pagado donde se
dijo, comprando que TODA la maquinaria de custodios de la suite —
fondeo, congelación, rotación, recuperación, pendientes, cupos y
contadores — ejercite la vía real en cada pasada.

**Lo que la campaña encontró por el camino**: el tope que vive en la
generación (contrato vía-recibo, §166); **la puerta que NO estaba**
(la delegada aceptaba same-identity: cura en el apply, §169 — la
primera divergencia real de guardas, destapada por un relevo que
nació en rojo); el recibo que cruza el reinicio (§169); §54.2 en vivo
(§168); el flake del cerrojo de sled (§165); y las reglas del rito:
censos sin head, colas envueltas, el detector de reuso como juez que
dos veces supo más que el censor. **B-3 queda armada**: jubilar las
cinco `#[deprecated]` con el libro como mapa — cada muerte,
relevada — y con ella el ✅ entero de la entrada 32.

## 171. B-3 CERRADA — la vía-recibo de custodios, retirada; la 32 y la 33, ✅

**La jubilación**: B-3a-i diecisiete lápidas · B-3a-ii seis adaptaciones
a la era delegada · B-3b-i la capa sin llamadores · B-3b-ii LA RETIRADA
(cinco funciones, cinco applies, cinco recibos, dos tipos Auth, cinco
circuitos, dos lápidas de two_phase; `d19b20e`, −990 líneas) · B-3c este
cierre (cuatro allows huérfanos fuera; los dos de tests reetiquetados —
aman `open_account`, no la vía muerta —; los dos ✅). Suite 252 → 231,
silencio 0·0.

**Los tres oros**, de guardas que nacieron para descubrir: la PUERTA QUE
NO ESTABA (la recuperación delegada aceptaba same-identity con Ok(); cura
en el apply, §169) · el commit huérfano (persistía la raíz sin el
registro — nodo irreiniciable tras recuperar, §169) · la forja aviso≠prueba
de §73.4, que la delegada vuelve estructuralmente INEXPRESABLE. Y un flake
de cerrojo con primo en dos crates (§165).

**La 32 y la 33, cerradas** — las que dieron nombre a media campaña. Ya
no existe camino por el que una clave de custodio llegue al operador: no es
una marca que ruega no usar la vía mala, es que la vía mala NO existe.
**Reglas del rito ganadas bajo fuego**: lápidas doble-asertadas; un `use`
de bloque se poda POR SÍMBOLO, nunca por línea; el caminante respeta la
frontera-de-blanco; la poda de un ayudante se censa por el ayudante; los
allows se retiran solo si su deprecated murió (los de tests amparaban
`open_account`, vivo); y la compuerta del silencio (cero warnings o no hay
commit) salvó la retirada dos veces. **Queda B-4**: medir (11) y canonizar
el rango de suite; luego el cierre de la Campaña B.

## 172. B-4 — la medida 11 EJECUTADA sobre el estado final, y la Campaña B CERRADA

**El canon de la capa, re-medido tras la migración** (release, protocolo
§89.1/§130, tres corridas, dispersión <2%): arranque **~1,0 ms**; emisión
2-de-N **~238 ms** generar / **~3,2 ms** aplicar / **54.667 B**;
transferencia (send+claim) **~380 ms** generar / **~68 ms** aplicar /
**132.311 B**; destrucción **~401 ms** / **~32 ms** / **62.290 B**; banda de
auditoría **~48 ms** / **~1,3 ms** / **50.933 B**. Lecturas estables:
verificar/generar de un pago completo **~18%**, auditar/transferir **~13%**,
jornada de mil pagos **~380 s** de prueba y **126,2 MB** acumulados. Los
tamaños de prueba no cambiaron con la migración — la vía delegada altera
quién autoriza, no la forma del ascenso — y así lo confirma el instrumento
girado (`metrics_of_the_layer`, hoy sobre la vía real).

**El rango de suite del estado final** (231 tests, release, tres pasadas):
**34,9 – 40,6 s**. Contra la proyección de §162 (~+52 s de techo sobre los
19-27 s del modo viejo ⇒ 70-80 s), el coste real fue **~+13-15 s**: la
proyección sobreestimaba porque asumía que cada uso migrado ejecuta una vez
por pasada, y la suite comparte capas entre asertos. El precio de la vía
real, ya con el número final en la mano, es la mitad de su techo — y se
sigue pagando donde se dijo, en máquina de CI, no en usuario.

**Campaña B, CERRADA.** El arco completo: censo canon (§160) → la palanca
y su paridad (B-0a) → el cadáver `transfer` fuera (B-1) → el precio medido y
la decisión por principios: RETIRAR (§162-§163) → la migración de las cinco
familias, test a test, con libro (§164-§170) → la jubilación de la
vía-recibo y sus dos ✅ (§171) → esta medición (§172). Trece asientos, una
suite que pasó de depender a medias de una vía marcada-sin-nombrarla a
ejercitar la vía real en cada pasada, tres oros de seguridad hallados por
guardas que nacieron para descubrir, y una fuga —la que el proyecto nació
para cerrar— que ya no tiene camino. **Lo que queda son frentes nuevos**,
no deudas de esta campaña: la medida 9 (Campaña C, «qué aprende cada
participante»), las notas/UTXO (6), el anclaje externo (10), la retirada
del museo, y las costuras ajenas anotadas (shim 49-A, costura 52).

## 173. Campaña C — la medida 9 estaba HECHA y dispersa; este asiento la reconoce y la cierra

**El hallazgo de apertura**: la medida 9 del triaje («qué aprende cada
participante», la que el triaje llamó «la mejor») no había que
implementarla desde cero — ya vivía en el árbol, con más rigor que la
lista, y sin asiento que la nombrara. Bajo la cabecera «MEDIDA 9» de
client.rs hay ONCE tests discriminantes de correlación, con la filosofía
correcta escrita en su encabezado: no describen el modelo, cazan las
fugas que sobrevivirían a un cambio de diseño — cada uno FALLA si la
propiedad de privacidad se rompe.

**Los tres roles, cubiertos donde tienen sentido — en el PAGO**, que es
la única operación de tres partes (emisor, receptor, tercero-observador).
**Tercero**: `el_commitment_no_revela_al_emisor` (dos pagadores distintos
al mismo receptor con mismo salt e importe producen commitments
idénticos), `el_salt_es_lo_que_hace_impagos_inenlazables` (la
unlinkability descansa en el salt, y su reutilización la quita),
`la_posicion_del_pendiente_es_orden_no_identidad`. **Contraparte** (el
receptor): `el_receptor_no_aprende_al_emisor_del_aviso` — el
`PendingNotice` no tiene campo `sender`, fijado en el tipo. **Materiales**
(la vía delegada de pago): `send_materials_contain_no_keys`,
`send_materials_alone_are_not_enough_to_spend`,
`a_whole_payment_without_giving_any_key_to_the_layer` (y su variante de
256 bits), `prove_send_rejects_a_key_that_is_not_the_holders`. Y el par
t5a/t5b, que mide el coste de la contención del anclaje global.

**El cuarto rol, el OPERADOR, es límite DECLARADO, no discriminable**:
qué aprende la capa no se prueba con un test que falle — se declara y se
acota. Su tesis vive en `commitment.rs` («la capa guarda (identidad,
saldo, nonce); NO lo necesita — el operador no puede leer nada del
compromiso») y en `crypto.rs` («NO protege contra el operador del nodo,
que ve los saldos»). Es honestidad de §16 hecha arquitectura: la capa por
compromisos demuestra que el operador PODRÍA no ver, y el modelo actual
declara que ve. Un discriminante aquí no aplica; el asiento lo fija como
frontera conocida.

**Un test de la medida vive huérfano, y se reconoce en su sitio**:
`nothing_leaks_before_authority_is_checked` (tests.rs) es rol-tercero
puro sobre la congelación — un intruso sin la clave recibe «no eres el
titular», NO «está congelada», así que no puede deducir el estado de
freeze de una cuenta ajena. Es medida 9 aplicada a las operaciones de
custodio (freeze/recovery/governance), que por lo demás son actos
custodio↔capa SIN contraparte ni tercero que cobre o reciba — por eso no
generan su propia batería. Se deja donde está (prueba orden-de-verificación
en su contexto de autoridad); este asiento es el hilo que lo une a la
campaña, sin moverlo.

**Campaña C, CERRADA por reconocimiento (✅ medida 9)**. Lo que el
triaje marcó «IMPLEMENTAR» estaba implementado; lo que faltaba era el
asiento que lo declarara medido. La suite lo ejercita en cada pasada (los
once discriminantes + el huérfano, todos en verde dentro de los 231). No
hay código nuevo: como en otras medidas del triaje, el proyecto lo había
hecho con más rigor que la lista, y la deuda era de registro, no de
mecanismo. **Del triaje quedan las documentales** (7 dinero-cuántico, 10
anclaje externo, 12 SECURITY, 13 posicionamiento — por errata §135 donde
toquen preprints) y los frentes mayores (6 notas/UTXO, museo). La medida
9 sale de la cola de pendientes.

## 174. Medida 10 — el anclaje externo, diseñado: el reloj que ni el operador ni el emisor controlan

**Qué pedía la 10**: BACKLOG y SECURITY §2.bis la nombran «interfaz de
anclaje externo (por diseñar)» — la *eliminación* del residuo #1 (orden y
completitud) que el acuse (§121) solo *acota*. No es código: es el
documento de diseño que faltaba, hermano de `CONFIANZA_RESIDUAL.md`. Vive
en `doc/ANCLAJE_EXTERNO.md`.

**El mecanismo, en una línea**: publicar la raíz de recepción —que ya va
comprometida y firmada en cada `EpochHead` (§121.2)— en un medio
append-only independiente del operador. Hereda la firma (cero índices
XMSS extra, la corrección que salvó al acuse) y admite anclaje por lotes
(raíz de un árbol de cabezas cada *M* latidos; el ancla individual es un
camino Merkle). *M* es una línea sistémica como `N_max`: se declara y se
mide.

**Lo que ataca, exacto**: los dos puntos que `CONFIANZA_RESIDUAL §8`
dejaba abiertos — la **vista dividida** (el operador firma cabezas
distintas a testigos distintos; el ancla la vuelve autodelatora sin
depender del gossip que CT infradesplegó) y el **RPO del siniestro** (deja
de depender de la cooperación del operador). Añade el tercer vértice a la
simetría §119/§121: un reloj que ni el operador (techo `N_max`) ni el
emisor (suelo) controlan.

**Encaje y honestidad**: dos filas nuevas en el Backlog de CONFIANZA —
`B10.6` (formato del ancla + *M*) y `B10.7` (verificador en cliente),
ninguna toca circuito, ambas sobre las cabezas que ya existen. El
documento nombra su propio residuo (§6): el medio externo es una
dependencia de confianza **desplazada, no eliminada**; la cola entre
anclas sigue acotada por *M* y N2, no cerrada; y el ancla prueba *qué
raíz* era canónica, no que su contenido fuera completo (eso sigue siendo
recibos + no-inclusión, §121.3). SECURITY §2.bis pasa a «interfaz
diseñada; pendiente de despliegue». **La 10 sale de la cola de
pendientes**; su primer paso medible es `B10.6`. Del triaje quedan las
documentales 7/12/13 y los frentes mayores (6 notas/UTXO, museo).

## 175. El frente del museo, ABIERTO con su mapa — y por qué sigue sin fecha, ahora con razón medida

**Qué se censó** (lectura sobre el árbol, sin tocar): el «museo»
`stark_experiment::circuit_settlement` (2.393 líneas) tiene una frontera
limpia entre DIECISÉIS helpers vivos —`derive_public_id{,_wide}`,
`native_leaf{,_salted}`, `native_nullifier{,_wide}`, `native_climb`,
`derive_leaf_salt{,_wide}`, `derive_view_key{,_wide}`, `view_id_of{,_wide}`,
`view_id_from_view_key`— concentrados en dos zonas (líneas 257-386 y
2084-2163), y el AIR MUERTO de la vía de un paso (§36):
`SettlementAir`, `SettlementProver`, `SenderWitness`, `ReceiverWitness`,
`SettlementPublicInputs` y sus `build_trace`, que son el grueso entre
medias.

**El hallazgo que MANTIENE la retirada sin fecha** —y corrige la
lectura ingenua de «fichero muerto, bórralo»: el AIR presunto-muerto NO
está muerto fuera de stark. `SettlementProver` tiene **nueve usuarios
vivos** (incluidos `zk-core` y `settlement-layer`), y `SenderWitness`/
`ReceiverWitness` viven en `settlement-layer/src/lib.rs`. Hay al menos
TRES crates en juego —`zk-ssl`, `zk-core`, `settlement-layer`— y DOS
ficheros `circuit_settlement.rs` (stark 2.393 líneas, zk-core 605). El
museo no es un fichero: es un nudo multi-crate.

**Las preguntas que el frente debe responder ANTES de mudar nada** —
este es el primer paso real, no la mudanza: (1) ¿qué es `settlement-layer`
frente a `zk-ssl` — dos capas, una obsoleta, o una dependencia? (2) el
`SettlementProver` que usan `zk-core` y `settlement-layer`, ¿es el museo
de stark o un homónimo en otra crate? (3) ¿por qué DOS
`circuit_settlement.rs`, y cuál es canónico? (4) el hogar de los
dieciséis helpers —`native_leaf` tiene **27 usuarios** repartidos entre
crates— ¿es `zk-core` (parece el núcleo), un módulo `native` nuevo, o
`rescue_hash` (el único candidato existente)? Sin estas cuatro
respuestas, cualquier plan de mudanza inventa el mapa.

**El orden, cuando el mapa esté claro**: censo helper-por-helper con
usuarios por crate → decisión del hogar (asiento propio) → mudar los
helpers y reapuntar imports (la palanca de precondición de §158) →
solo entonces retirar el AIR muerto, si de verdad quedó sin usuarios
vivos tras aclarar el nudo de `SettlementProver`. Es un frente del
tamaño de una tanda B, con su riesgo de anclas entre crates; se hace con
la mesa despejada y espejo fresco, como §158 ya intuía al no ponerle
fecha.

**Las dos costuras, que NO son de este frente ni de nadie ahora** —
registradas para que no se confundan con trabajo pendiente: la costura
**49-A↔52** (recovery.rs:93, el view_id/salt que la rotación deja viejo)
es diseño de la **entrada 52**, no de 49-A —«la capa no puede
recalcularlo: no tiene la clave», limitación declarada con test
`recovery_deja_view_id_viejo`, no agujero—; y el **shim 49-A**
(`record_to_bytes`) migra con el cierre de 49-A (§158). Ninguna se toca
sin abrir su entrada dueña; forzarlas sería entrar por la puerta de
atrás. Están donde deben, marcadas y con test.

## 176. El frente del museo, CERRADO — M-1/M-2/M-3: la fragua vive, el museo cayó

**Las cuatro preguntas de §175, respondidas del natural antes de cortar**:
(1) `settlement-layer` es la capa del mundo Groth16/arkworks anterior
(sobre `zk-core`); `zk-ssl` es la actual, STARK, y jamás toca aquel mundo.
(2) El `SettlementProver` «con nueve usuarios» era doble homónimo: el
nombre del TRAIT de `settlement-prover` (que los tres backends implementan
para `compliance_circuit`, no para el museo) y los tipos Groth16 propios
de zk-core — el struct del museo tenía CERO usuarios en todo el workspace.
(3) Dos `circuit_settlement.rs` = dos mundos de prueba, cada uno canónico
en el suyo — y el homónimo mordió una última vez: el censo del derribo
tropezó con el `pub mod` de zk-core (vivo, legítimo) y se corrigió a
ámbito-STARK. (4) El hogar: `stark-experiment/src/native.rs`, nacido en
M-1.

**Las tres operaciones**: **M-1** (`05df78a`) mudó LA FRAGUA verbatim por
caminante — 16 helpers, 3 constantes de dominio y el utilitario privado
`as_digest` que el compilador nombró — con el museo re-exportando por
compatibilidad y sus tests enjaulados recibiendo suministros cfg-gated
(lección: los warnings de la lib no absuelven a los tests). **M-2**
(`45f0eca`) reapuntó 51 referencias en 24 ficheros (15 stark-internas, 36
del lado zk-ssl) y degradó el compat a glob privado. **M-3** derribó el
fichero entero (2.230 líneas ya sin fragua) y su `pub mod`: cero código
del mundo STARK nombra al muerto; las menciones en prosa quedan como
registro.

**Los números del cierre, predichos por SIMULACIÓN en réplica antes de
cortar y clavados al primer golpe**: stark 316/0+12 → **289/0+10** —los
27 del museo que corrían en release mueren con él, sus 2 `#[ignore]`
(instrumentos de medida) también—; el guardián de layout 27 → **26
circuitos** —el `SettlementAir` era uno—; zk-ssl intacto en **231/0+3**
porque toda su dieta era la fragua, que vive. La condición de §158 —«los
helpers nativos se mudan ANTES»— se cumplió al pie: **el museo se fue, la
fragua se quedó**. La simulación-en-réplica pre-derribo entra al rito. El
frente sale de la cola; quedan las documentales 7/12/13 y la medida
6 del triaje (notas/UTXO) — que §177 examina y resuelve.

## 177. La medida 6 (notas/UTXO), EXAMINADA frente a tres ataques — y por qué el modelo de cuentas los cubre sin rediseño

**Enmienda de registro primero**: la «entrada 6» del BACKLOG (grado
dependiente del testigo, «declarar no migrar», §46) está ✅ cerrada y es
OTRA cosa; lo que el triaje numera **6** es «notas/UTXO → arquitectura
mayor». Los traspasos conflaban ambas numeraciones; queda enmendado aquí
y en §176. Y la bifurcación cuentas-vs-notas no se decide desde cero: **la
tabla de Salidas de §4** ya pesó el modelo UTXO —«compatible con los
principios: Sí; coste: rediseño del sistema entero»— y eligió dos-fases,
que la casa hoy ejecuta, mide (§172) y guarda (§173).

**El examen — tres ataques que UTXO tocaría, medidos contra lo que la
capa hace hoy**:

**(1) El ataque del vecino** (leer el saldo ajeno por el camino de Merkle
del árbol de cuentas, eligiendo quién espiar al cronometrar las altas):
**cerrado en el modelo de cuentas, sin UTXO**. La colocación NO es
`next+1` — es `public_id[0] mod capacidad` con sondeo lineal
(accounts.rs:207), así que el índice deriva de la identidad, que deriva de
la clave de gasto (§127). Un atacante no elige vecino: necesitaría una
preimagen que aterrice contigua, no un reloj. El test
`account_indices_are_not_predictable` lo verifica en verde. UTXO lo
cerraría por construcción (no hay hoja de saldo por identidad); el modelo
de cuentas lo cerró por colocación. **Empate — ambos lo resuelven.**

**(2) El pago no reclamado / inmovilización**: dos-fases lo tiene —no
existe operación de cancelar, caducar ni reembolsar un pendiente; sin el
`claim` del receptor, los fondos quedan quietos—, y **es coste DECLARADO,
no defecto oculto** (§29-§30 «el coste declarado de las dos fases», entrada
12, asumido en ESCALADO.md; y es MENOS limbo que la vía de un paso que
sustituyó, §7/§36). UTXO clásico tiene el mismo defecto base y a veces
trae time-locks/reembolso —salidas de diseño que aquí no se
implementaron—. **Único frente donde UTXO ofrecería algo; pero se cierra
con una operación de caducidad sobre el modelo actual, NO con un rediseño
a notas.**

**(3) El ordenamiento distribuido / consenso**: **ninguno lo resuelve, y
es ortogonal al modelo de datos**. Quién ordena las operaciones es capa
distinta de cómo se representa el valor —Bitcoin es UTXO *y* PoW porque son
cosas separadas—. La casa lo declara fuera de alcance (`CONFIANZA_RESIDUAL
§8`: «el operador sigue ordenando; consenso sigue fuera de alcance») y el
anclaje externo (§174) solo lo acota. UTXO no cambiaría una coma.
**Empate — ninguno; irrelevante a la bifurcación.**

**El veredicto**: dos-fases **no domina** a UTXO —empata en vecino y
consenso, y en inmovilización UTXO ofrece salidas que aquí faltan— pero
**UTXO tampoco domina**, y su único frente ganador (la inmovilización) se
cierra con una operación de caducidad sobre cuentas, sin pagar el rediseño
entero que §4 rechazó y que rompería cuentas, congelación, recuperación,
gobernanza, banda de auditoría y mapeo ISO. Lo que UTXO compraría de más
—ceguera del operador— ya tiene dueño mejor (B11, `CONFIANZA §3`). **La
medida 6 no se persigue como rediseño**; su única deuda viva y localizada
—la caducidad del pendiente inmovilizado— queda anotada como mejora sobre
el modelo actual, no como razón para notas. — y **DISEÑADA en §178** (`doc/CADUCIDAD_PENDIENTE.md`). Si la bifurcación se reabriera,
sería **contra esta acta**. **La cola mayor del triaje queda VACÍA**: solo
restan las documentales 7/12/13.

## 178. La caducidad del pendiente, DISEÑADA — el reembolso del emisor tras T, con doble cerrojo

**Qué cierra**: la única deuda viva que el examen de §177 dejó — el
frente donde el modelo de notas ofrecía algo (time-locks) que dos-fases no
tenía. El diseño vive en `doc/CADUCIDAD_PENDIENTE.md` (hermano de
ANCLAJE_EXTERNO) y se construye entero sobre piezas EXISTENTES:
`apply_send` ya recibe `sender_index`, `pending_amounts` ya sienta el
precedente del metadato-declarado del operador, `epoch_head().seq` ya es
reloj, y el crédito reusa `mint_climb` sin tocar una coma.

**El corazón del diseño es el ataque que lo mata en su forma ingenua**:
un refund que solo exija conocer la apertura del compromiso vuelve ROBABLE
el aviso filtrado (hoy inocuo: `claim` ata la clave del receptor, §39.1).
El doble cerrojo lo cierra: (1) el DESTINO lo fijan los registros — la
capa acredita a `meta.sender_index` pruebe quien pruebe, así que el
ladrón, como mucho, le paga el trámite al emisor; (2) los MATERIALES solo
puede fabricarlos el emisor — la subida de crédito exige el preimagen de
su hoja, cuyo salt deriva de su clave (§117). Ni el compromiso ni `claim`
ni los discriminantes de §173 se tocan: el emisor entra en los registros
del operador (que ya lo conocía, §21/§129), jamás en el árbol.

**Las decisiones con nombre**: `T` es línea sistémica declarada (familia
`N_max`/§121 y `M`/§174), que hereda el reloj de cabezas firmadas cuando
existan (§115); la CARRERA post-T entre claim y refund se declara como
semántica —un parámetro parte el tiempo en dos derechos, el primero en
aplicarse gana—; los pendientes de EMISIÓN se DES-EMITEN al caducar
(simetría: la emisión no cobrada deja de existir); la clave-perdida del
emisor va a fase-2 remanente (familia B18.3, decisión política); y el
operador puede acelerar devoluciones pero nunca desviarlas — declarado. El
meta viaja EN EL LOTE del commit, con la lección del oro de §169 escrita
como test desde el primer día.

**Coste y frontera**: un circuito pequeño nuevo (`circuit_refund`,
clase claim-sin-identidad: apertura + vaciado; sin PK, sin nullifier —la
hoja vacía es su anti-replay—, sin frozen; el guardián de layout pasará de
26 a 27), una compuerta de capa, el meta persistido, y una batería de
tests que incluye al ladrón-con-aviso y la persistencia tras reinicio.
**Primer paso medible**: el circuito con su layout y sus discriminantes,
ANTES de tocar la capa. Es trabajo que estrena restricciones:
**sesión propia**, como la casa manda — este asiento deja el diseño
completo para que esa sesión ejecute, no explore.

## 179. R-1 EJECUTADO — el circuito #27 (`circuit_refund`) nace y verifica al primer vuelo

**El primer paso medible de §178, hecho** (`f2b3ca9`): la apertura del
compromiso pendiente en la traza más corta de la casa — 16 filas, una vía,
dos merges de Rescue. Veinte restricciones (`C_HASH` 12 rondas, `C_CAP` 4
capacidades, `C_CARRY` 4 portes del digest interior) y doce aserciones que
dejan al receptor y al salt LIBRES como testigo e inyectan el importe
(fila 8) y `P` (fila final) como públicos. Cuatro tests, dos de ellos
DISCRIMINANTES: otro importe NO verifica, otro compromiso NO verifica. Un
AIR nuevo contra winterfell que compiló, probó y verificó de estreno; el
único tropiezo fue un `use` sobrante que el compilador nombró. **Canon
nuevo: stark 293/0+10 · guardián 27 circuitos**; zk-ssl intacto en
231.

**El reparto circuito/capa, sellado en el diseño**: el circuito lleva el
medio-cerrojo criptográfico —la apertura ata importe y compromiso, así que
el ladrón no elige cuánto ni transfiere la prueba a otra hoja— y
deliberadamente NO lleva PK (el destino lo fija la capa desde
`pending_meta`), ni nullifier (la hoja vacía es su anti-replay), ni frozen
(devolver no es cobrar), ni camino (`P` ya es público: la capa comprueba
`hoja[pos]==P` y vacía nativamente, como todo apply). El otro
medio-cerrojo —destino-por-registros y materiales-solo-del-emisor— es de
capa, y es exactamente R-2.

**R-2, con su orden censo-primero ya escrito**: (1) leer del natural
`apply_send`/`apply_claim` (el ancla del meta y el patrón de vaciado), la
forma del COMMIT EN LOTE y la persistencia — la lección de §169 exige que
`pending_meta` viaje en el lote, lo que probablemente estrena una versión
de snapshot; (2) meta `{sender_index, born_seq}` + compuerta TTL sobre
`epoch_head().seq` + `apply_refund` cosiendo `circuit_refund` con el
`mint_climb` reusado; (3) la des-emisión de mint-pendientes; (4) la
batería de punta a punta — el ladrón-con-aviso rebotando por sus dos
cerrojos, la carrera post-T, y el reinicio que conserva el meta; (5)
medición. Es la mitad grande y toca persistencia: **sesión propia, con
censo fresco** — este asiento es su punto de partida.

## 180. R-2a y R-2b EJECUTADOS — el meta en el lote y el gemelo del clímber; los dos circuitos de la caducidad, vivos

**R-2a** (`da583fe`): la infraestructura del meta — `pending_meta
{emisor, nacimiento}` nace en `apply_send` y en mint-a-pendiente (con
CENTINELA `u64::MAX`: des-emisión, no reembolso), muere en `claim`, y
**viaja EN EL LOTE** junto a `pamt:` (la lección de §169 cumplida:
`pmeta:` de 16 bytes, vacío = sin-meta). `T = refund_ttl` queda como línea
sistémica (DEFAULT 64 latidos de `log.seq`, knob, persistida). **Legado
declarado**: pendientes pre-R2a sin meta son INMUNES a la caducidad —
ausencia es inmunidad, no fabricación. La tanda pagó tres reglas nuevas al
rito: un ancla-de-fn CARGA con su atributo (la compuerta del silencio cazó
un `#[test]` desnudado — un test existente silenciosamente desactivado, su
mejor presa); los bloques de reparación se escriben IDEMPOTENTES; los
constructores de un struct se censan TODOS.

**R-2b** (`0e5a933`): el censo FALSIFICÓ el reuso de `mint_climb` con
`delta=0` — su `C_SUPPLY` ata `supply_new − supply_old == importe` («EL
SUMINISTRO SUBE EXACTAMENTE EN EL IMPORTE»); `supply_delta` existe para
que la emisión encubierta se RECHACE, no para esquivarla. La respuesta
honesta: **`circuit_credit_climb`**, gemelo con TRES cortes marcados en el
código (sin `C_SUPPLY`; rango de 3 segmentos, no 5; publics sin
suministro) y cabecera-puntero al origen para que el drift grite (lección
del museo). Dos discriminantes: otro importe NO verifica, y **una subida
con salt AJENO no verifica contra la raíz del titular** — el cerrojo #2 de
§178 (§117), probado en circuito. Una costura al vuelo (aserciones
24→23). **Canon: stark 297/0+10 · guardián 28 circuitos · zk-ssl
234/0+3.**

**R-2c, el tejido, con su guion censo-primero**: (1) **censo de
`OpKind`** ANTES de tocarlo — enum con `tag_byte`/`from_tag_byte` y
matches exhaustivos repartidos por el árbol; añadir `Refund` sin censar
cada match es rotura en cadena; (2) `RefundReceipt` + `refund()` del
emisor (deriva su salt de la clave, §117; arma las DOS pruebas); (3)
`apply_refund`: compuertas de capa —meta existe y no-centinela,
`seq − nacimiento ≥ T`, `hoja[pos] == P`, `amounts[pos] == importe`— más
verificación de ambas pruebas con los publics construidos por la capa
(`root_new` del tentativo que acredita a `meta.sender_index`: el destino
atado por raíces, patrón del delegado), mutación, `OpKind::Refund` al log,
`commit(&[emisor], Some((pos, vacía)))`; (4) la batería: feliz, pre-T
rebota, **ladrón-con-aviso** (su clímber no casa la raíz del emisor),
carrera post-T, replay (hoja vacía), y la persistencia del meta con el
patrón de reapertura leído del natural; (5) **R-2d**: des-emisión del
centinela (sin clímber — no hay cuenta; `total_supply` BAJA, que subió al
emitir), medición, y el asiento de cierre. Sesión propia: es el tejido
donde los dos cerrojos se cierran en la capa.

## 181. §178 EJECUTADO — la caducidad completa, con sus números

**R-2c, el tejido** (`1d244d2`): `refund()` espeja a `send()` guarda por
guarda —titularidad, y la hoja reconstruida con el salt DERIVADO de la
clave (§117): clave ajena, hoja ajena, rebote— y fabrica las dos pruebas;
`apply_refund` pasa las compuertas (meta existe y no-centinela →
`RefundUnavailable`; `seq − nacido ≥ T` → `RefundTooEarly`; hoja == P e
importe == registrado → `PendingMismatch`) y verifica AMBAS con publics
construidos por la capa: el `root_new` del tentativo que acredita a
`meta.sender_index` — **el destino atado por raíces**, patrón del
delegado. `OpKind::Refund` (tag 12). Cinco tests: feliz, pre-T, **el
ladrón con el aviso entero rebotando** (su clímber no casa la raíz que
acredita al emisor: los dos cerrojos mordiendo juntos), **la carrera en
ambos órdenes**, y el meta sobreviviendo al reinicio.

**R-2d, la des-emisión** (`0dcc6bc`): los mint-pendientes del centinela
caducan DESTRUYENDO — `total_supply` baja exactamente lo que subió al
nacer, verificado al satoshi; sin clímber (no hay cuenta que acreditar),
solo la apertura #27 prueba que se destruye lo comprometido. Y **las vías
no se cruzan**: el centinela no admite crédito a nadie, el pago no se
destruye — cada caducidad por su puerta, ambas rebotando
`RefundUnavailable`.

**Los números** (Ryzen del piloto, `--release`): reembolso completo
**~203 ms** de generación + **~10 ms** de aplicación, con **14.257 B** de
apertura + **54.654 B** de crédito (~69 KB); des-emisión **~189 ms** +
**~7,6 ms** con **13.759 B**. Del orden del resto de la casa
(transferencia ~380 ms / 132 KB): la caducidad no es la operación cara.

**Dos lecciones del tejido, al rito**: (1) el censo de un enum incluye
TODOS sus matches exhaustivos, y viven donde el grep temático no mira —
`iso.rs` traduce `LayerError` a códigos ISO SIN catch-all *por diseño*
(se eliminó porque absorbía errores en silencio), y solo el compilador
sabía que mis tres errores nuevos necesitaban sus códigos pensados
(DS0G/AG08); (2) un rojo puede ser un HALLAZGO: en el Orden A de la
carrera, el cobro se lleva el meta consigo y el reembolso ya NI CALIFICA
(`RefundUnavailable`, la primera compuerta) — mejor comportamiento que el
que mi test esperaba; la expectativa se corrigió hacia el código, no al
revés. **Estado de §178: EJECUTADO.** Canon: stark 297/0+10 · zk-ssl
242/0+3 · guardianes 39 · 28. Queda de su cola solo prosa: las
documentales 7/12/13 del triaje.

## 182. Documentales 7/12/13 ejecutadas — y el triaje de las 14 medidas, CERRADO

**7 — dinero cuántico**: sección propia en README con la distinción que
importa: lo que este proyecto YA es (solidez post-cuántica: STARK/FRI y
autoridad-por-preimagen §117, sin firma clásica en la vía de pago; XMSS
evaluado) y lo que NO es (dinero cuántico de verdad: no-clonabilidad
física que retiraría al intermediario de orden — el horizonte de
PRINCIPIOS §6.bis del que esto es la aproximación clásica). **12 —
SECURITY**: §1 gana la caducidad con su doble cerrojo y sus tres tests
nombrados como testigos; §2 gana dos honestidades — la carrera post-T es
del operador (residuo de orden §121, no superficie nueva) y la vía
retirada solo sabe inmovilizar (§177). **13 — posicionamiento**: ya
existía de una era anterior; refrescado (la caducidad entra al inventario;
la línea de BACKLOG 32/33 pasa de «pendiente» a «retirada ✅»).

Con estas tres, **las 14 medidas del triaje quedan todas resueltas**:
ejecutadas con más rigor que lo pedido (1/2/3b/8/9/10/11/7/12/13),
rechazadas con acta (3a→§129, 4b/14→§135, 4a→era la 32), o promovidas a
arquitectura con sesión propia (6→§177 la examinó contra tres ataques).
El triaje nació para ordenar; muere ordenado.

## 183. Crítica externa: verificación formal — triaje, dos verdades y cuatro medidas

Llegó una crítica externa: los circuitos sub-restringidos son el riesgo
número uno en ZK, y sin verificación formal de los layouts no hay
soberanía que valga. **El triaje le da la razón en el núcleo y le corrige
el mapa.** La razón: la suite prueba PUNTOS, no universales — 297+242
verdes no demuestran que ningún testigo-fantasma exista; el §69 pedía este
residuo con nombre y ahora lo tiene (SECURITY §2). El mapa: aquí no hay
R1CS ni Plonkish (es AIR/winterfell, Goldilocks, sbox grado 7 — las
herramientas de estante son del mundo circom y no aplican);
`circuit_settlement.rs` ya no existe (§175); y el guardián de layout YA
cubre la mitad sintáctica de la clase, con casos-mutación sembrados y
bugs reales cazados en su acta de nacimiento.

**Cuatro medidas** (`doc/VERIFICACION_FORMAL.md`): **FV-0** esta tinta;
**FV-1**, la implementable — el CENSO DE CELDAS en el guardián: cada
columna × clase-de-fila con dueño (transición activa, aserción, o
testigo-libre DECLARADO), con su caso-mutación («borrar un C_CAP debe
gritar»), y la honestidad en su propia salida: referenciada ≠ determinada;
**FV-2**, spike SMT acotado a circuit_refund (cvc5 cuerpo-finito) con
permiso explícito para concluir «intratable, con acta»; **FV-3**, Lean/K
como horizonte nombrado — el trato del dinero cuántico, no una promesa.
Y una precisión de especificación para no verificar el teorema
equivocado: la propiedad no es «todo testigo da el mismo nullifier»
(falsa: usuarios distintos, nullifiers distintos) sino determinación
funcional por (clave, dominio). FV-1 es sesión propia y empieza por
censar al guardián mismo: 600+ líneas que se extienden tras leerse
enteras, no tras adivinarse.

## 184. FV-1: el prototipo ejecutado convierte la estimación en requisito — y una corrección de acta

Se desarrolló el primer paso de la entrada 71 hasta un prototipo funcional
sobre `circuit_refund` y **se ejecutó** (el método de la casa: no se
registra un enfoque sin correrlo). Hace lo que el guardián de ranuras no
hace —parsea transiciones y aserciones y cruza por celda contra celdas
libres declaradas— pero su **caso-mutación, borrar `C_CAP`, NO se
distingue**: sin un intérprete que derive «clase de fila = filas donde el
selector vale 1», la clase agregada no aísla las filas donde la restricción
borrada era el único dueño. **El valor**: §183 estimó FV-1 como «extender
el guardián»; la ejecución lo corrige a un requisito probado — el núcleo de
FV-1 es **el intérprete de selectores periódicos**, no el cruce (la parte
fácil, ya prototipada). El prototipo vive en `doc/fv/`, no en `tools/`: no
caza su mutante, luego no es guardián (§42.5).

⚠️ **Corrección de acta (§111.2 en carne propia)**: el commit `0496b8b`
selló un mensaje que anunciaba este §184, la §6 del doc y la entrada 71
actualizada —y NINGUNA de las tres se había escrito—. La guarda del bloque
buscaba el prototipo con un nombre (`censo_celdas_prototipo.py`) distinto
del que el arrastre dejó (`censo_celdas.py`), abortó su rama por diseño, y
un `git add -A` de cola coló el fichero suelto mientras el commit describía
tinta inexistente. Es exactamente el fallo que §111.2 registró —un asiento
que narra lo que no pasó—. Este asiento existe de verdad, y con él las tres
piezas prometidas. Canon sin cambio (297 · 242 · 40 · 28).

## 185. FV-1 cruza la frontera — el intérprete caza el mutante, y la aserción tiene clase

El intérprete de selectores que §184 fijó como núcleo de FV-1 está escrito
y **funciona**: `doc/fv/interprete_selectores.py` deriva las clases de fila
del selector real de `circuit_refund` —hash = filas 0..6, enlace = fila 7—
y **distingue el mutante** que el prototipo no distinguía: borrar `C_CAP`
deja cuatro capacidades sin dueño en la clase «enlace».

**El hallazgo que lo desbloqueó** es de la clase de §72 —una garantía que
se creía puesta y vivía en la fila equivocada—. El prototipo trataba una
aserción como cobertura de su columna en TODA clase de fila; pero
`Assertion::single(c, 0, zero)` ata la capacidad `c` a cero SOLO en la fila
0 (clase «hash»), mientras `C_CAP` la gobierna en la fila 7 (clase
«enlace»). Cubrir toda clase con una aserción de una fila borra la
distinción que caza el sub-restringimiento —el mismo error de agregación
que §184 vio en las clases, ahora en las aserciones—. La aserción tiene
clase: la de su fila, y ninguna otra.

**Estado de la entrada 71**: el núcleo difícil (derivar clase-de-fila del
selector, ubicar aserciones por su fila, cazar el hueco) está probado sobre
el circuito simple. Queda generalizar a los 28 —selectores multi-ciclo,
carriles duales, aserciones en filas no-cero— e injertar en el guardián
como compuerta. El concepto está resuelto; el resto es cobertura. Canon sin
cambio (297 · 242 · 40 · 28).

## 186. FV-1 a dos carriles — y el falso positivo que casi se disfraza de hallazgo

El intérprete de §185 se generalizó al primer circuito de dos carriles
(`circuit_frozen_climb`) y **cumple el criterio**: sano cero huérfanas,
mutante cazado (cuatro capacidades del carril A). Cuatro piezas nuevas:
bucle de carril, índices crudos, selector booleano de fila-completa, y
seguimiento de aliases.

**El hallazgo es el más valioso de la serie FV, y es una advertencia sobre
las herramientas de verificación mismas.** La primera versión declaró
`COL_BIT` (col 24) sin dueño en el circuito SANO. No era sub-restringimiento
—era ceguera del intérprete—: `COL_BIT` se lee vía `let bit =
next[COL_BIT];` y el `bit` gobierna las colocaciones bajo `link_flag`; el
parser no seguía el alias. Un verificador sintáctico que grita
"vulnerabilidad" sobre código sano es peor que no tenerlo —destruye su
propia credibilidad, la lección de §42.5 aplicada a la herramienta en vez
de al circuito, y §137 (censar todas las representaciones) en su forma más
peligrosa: la representación no vista se confunde con un fallo—. Resuelto
con cosecha de aliases. Esta trampa habría hecho a FV-1, aplicado a ciegas
sobre los 28, reportar falsos sub-restringimientos que un revisor tomaría
por reales; queda documentada para que el injerto no reincida.

**Estado 71**: concepto probado en las dos topologías (un carril, dos
carriles) con la trampa del alias resuelta. Queda cobertura —selectores
multi-ciclo de los circuitos de cuentas— y el injerto en el guardián. Canon
sin cambio (297 · 242 · 40 · 28).

## 187. El mapa de tres capas — doc §9: la sesión propia de la 71 queda con plano

La crítica externa de FV quedó triada y respondida con tres intérpretes
ejecutables (§184-§186); lo que FALTA quedaba en prosa de traspaso, no en
canon. Este asiento lo fija: doc/VERIFICACION_FORMAL.md gana el §9 («El
mapa de lo que falta — tres capas»), leído del árbol, que corrige una
imprecisión de traspasos previos: el resto NO es «solo cobertura» — la
Capa 1 es un TERCER patrón de selector (cuatro familias: hash multi-ciclo,
one-hots de árbol, fila-0, segmentos Horner) que el intérprete de ciclo
corto no sabe leer. Capa 2, el injerto en el guardián (751 líneas, leídas
enteras antes de tocar). Capa 3, FV-2/FV-3 intactas detrás. Criterio
invariante desde §185: mint_climb sano cero huérfanas Y mutante cazado.
El contenido del §9 viajó también como doc/fv/mapa_fv_capas.md.

## 188. Capa 1 EJECUTADA — el intérprete multi-ciclo, 588 celdas-clase, y el candidato ciego

La sesión propia que §187 dejaba con plano. doc/fv/interprete_multiciclo.py
lee las CUATRO familias del §9 sobre circuit_mint_climb y asume de entrada
el riesgo declarado del mapa: clase = FIRMA explícita de selectores por
paso (0..510), no patrón de ciclo. Medido: 14 clases · 588 celdas-clase ·
33 sentencias result · 140 celdas libres DECLARADAS en el propio circuito
(convención CELDAS_LIBRES del doc §1: salt de hoja testigo, bit de camino
fuera de las filas de enlace, descansos del acumulador, limbos altos del
primer merge, y los carriles tras la raíz — la cola plana son 192 de los
511 pasos) · 0 rancias · 0 SIN DUEÑO. Mutantes: C_HORNER cazado (7
huérfanas nuevas: col 37 en las clases de cuerpo) y C_SALT_IN_A cazado (4
nuevas: cols 8..11 en la clase del tercer merge).

El HALLAZGO: el candidato del mapa (borrar una C_SEG_LINK) es CIEGO para
el censo — delta 0, porque Horner co-posee la col 37 en las clases de
cierre (cont_s cubre la penúltima fila de cada segmento). La red que lo ve
es el guardián, por partida doble y medido sobre copia mutada: [MUERTA] 5
ranuras sin escribir (grupo C_SEG_LINK) y [MUERTA PERIODICA] columnas
31..35 construidas sin leerse. Matiz que la Capa 2 hereda: ambas avisan
como HUECOS, no graves (exit 0); la compuerta del rito cae igualmente
porque la línea limpia del resumen desaparece. La cosecha del intérprete
es conservadora (alias de columna Y de selector, arrays de constantes con
deref, bucles reales por pila de llaves, ensanche del bloque de carril
solo para la forma-hash): lo no entendido jamás SUMA cobertura — el sesgo
es al rojo, §59.2 no entra por aquí. Dos cicatrices del camino: el locus
de CELDAS_LIBRES moría en el primer paréntesis de prosa (§117) — línea y
locus van ahora separados —, y la aserción de ROW_ACCT_ROOT acredita a
TODA la clase cont_s (39 filas) por doctrina §185: el propio censo lo
recuerda en cada salida — referenciada ≠ determinada.

Queda de la 71: Capa 2, el injerto (con este hallazgo en la mano: el censo
entra como compuerta que TUMBA, y la política huecos/graves de las MUERTAS
se mira ahí). Detrás, intacto, el §9.

## 189. Capa 2 EJECUTADA — el censo es compuerta del guardián y las MUERTAS son graves

El injerto que doc §9 pedía, hecho y medido. `tools/check_constraint_layout.py`
queda en 894 líneas: `main()` se parte en `barrer(raiz, verbose, con_censo)` y
`censo_mint_climb()` ejecuta `doc/fv/interprete_multiciclo.py` como subprocess
y EXIGE su COMPUERTA sana — exit 0, «0 sin dueño», «mutantes 2/2» (el
caso-mutación viene sembrado de fábrica: el intérprete caza sus dos mutantes
en cada corrida, y esta compuerta lo exige). Toda falta del censo es GRAVE,
incluida la ausencia del intérprete: una compuerta que se salta cuando su
herramienta no está es el falso verde de §59.2. La línea limpia conserva el
prefijo del rito y gana el censo: «28 circuitos: … sin leerse. Censo
mint_climb: 588 celdas-clase en 14 clases · 140 libres declaradas · 0 sin
dueño · mutantes 2/2.»

**La política que §188 dejó a mirar, decidida por la doctrina del propio
guardián** (condenar lo que no se entiende es tan malo como aprobarlo,
§42.5/§59.2): hallazgo POSITIVO medido = GRAVE; incapacidad del barrido =
hueco. [MUERTA] y [MUERTA PERIODICA] suben a graves — eran los huecos con
exit 0 en los que el candidato ciego se apoyaba —; toda la familia [?] sigue
siendo hueco. `CASO_188` queda sembrado en el autotest y pide el barrido
ENTERO, no `analizar`: una regresión de la política no puede pasar en
silencio. Autotest 3/3.

**Las tres cazas, medidas sobre copias del árbol** — la complementariedad es
bidireccional y MEDIDA, no supuesta: borrar `C_SEG_LINK` (el candidato ciego
de §188) deja el censo en delta 0 y son las redes quienes lo ven ([MUERTA] 5
ranuras + [MUERTA PERIODICA] [31..35], GRAVES; exit 0→1 — el candidato ya no
sale); cambiar el SELECTOR (`cont_s`→`first_s` en Horner) deja todas las
ranuras escritas y todas las periódicas leídas — redes CIEGAS — y el censo
grita 5 celdas-clase sin dueño (exit 1 — la compuerta ve lo que las redes no
ven); borrar `C_HORNER` dispara las dos con granos distintos (1 ranura muerta
contra 7 celdas-clase huérfanas). Regalo del diseño: la sonda interna del
intérprete reporta sola quién caza al candidato en cada baseline
(`C_SEG_LINK→guardián` en sano, `→censo` cuando Horner falta).

**Alcance honesto**: el censo cubre `circuit_mint_climb`, el único con
intérprete multi-ciclo; `circuit_frozen_climb` tiene el suyo (§186) en
doc/fv/ como evidencia; los hermanos de CUENTAS (`credit_climb`,
`recovery_climb`, `mint`, `send`, `claim`) siguen bajo las redes clásicas y
SIN censo — no hay falso verde: la línea limpia nombra su alcance. La 71
sigue ABIERTA en el BACKLOG con la extensión circuito a circuito como resto;
doc §9 y su gemelo `mapa_fv_capas.md` quedan marcados (Capas 1-2 EJECUTADAS).

## 190. FV-2 EJECUTADA CON ACTA — q2 unsat · q1 intratable por MEMORIA · el coste, comprado

El spike de doc §2, corrido entero y cerrado en sus propios términos. Dos
entregables en el árbol: `doc/fv/fv2_exporta_smt.py` (exportador SMT2 del
sistema de `circuit_refund`: 20 restricciones con selectores evaluados por
fila + 12 aserciones, traza 16×12; autotest interno que construye la traza
honesta con la semántica de `build_trace` y exige 0 en las 176 restricciones
activas; 9 anclas verbatim count==1 sobre el circuito; COMPUERTA-FV2 estable
en tres corridas con constantes reales: q1 385 asserts · q1m 383 · q2 97) y
`examples/volcado_rescue.rs` (las constantes MDS/INV_MDS/ARK1/ARK2 las canta
winterfell a JSON — verificadas MDS·INV_MDS=I antes de usarse). La
interpretación de q2 quedó DECLARADA (rito §4): cada ciclo de 7 rondas
abstraído como permutación no interpretada R compartida; y su cicatriz,
cazada en ensayo: la diferencia v1 barría las filas intermedias, sueltas a
propósito en la abstracción — SAT trivial sin significado; v2 acota a la
cadena {7, 8, 15}.

**Lo verde, medido**: `q2_cadena_uf → unsat` en 18/43/46 ms (tres corridas,
cvc5 1.3.2 pip — la igualdad sobre el sort FF no necesita CoCoA): la
fontanería (C_CAP + C_CARRY + aserciones) determina sola el estado de la
cadena, sin apoyarse en el álgebra de Rescue. La sintaxis SMT2 emitida
parseó entera al primer intento (el error de la sonda fue de capacidad, no
de forma).

**El muro, medido**: `q1_determinacion` y `q1_mut_carry` (el sistema séptico
completo, 336 ecuaciones de grado 7 sobre 384+5 símbolos) agotan la jaula de
6 GB en ~21 s y abortan — SIGABRT con «double free or corruption»: CoCoA no
muere elegante al fallar la reserva — IDÉNTICO en los dos modos del motor
(`gb` 20,9/21,6 s · `split` 21,0/22,0 s). Sin jaula, el segador de WSL los
mata por SIGTERM antes. No es timeout: es memoria, y llega en segundos. El
mutante corre la misma suerte, luego la caza SMT no se consumó: el
exportador SE QUEDA en doc/fv/ como evidencia (§42.5) — su autotest de traza
honesta y sus anclas vigilan la fidelidad de la traducción, no la solidez.
**Escalar a `circuit_send` (51 columnas): CERRADO con datos** — si 16×12
revienta 6 GB en 21 s, no hay pregunta que abrir.

**El coste, comprado, con sus cicatrices** (todas al traspaso): el cvc5 de
pip viene sin CoCoA («not configured with --cocoa» — GPL no viaja en
binarios); el build desde fuente exige `--gpl` explícito y `--prefix=` con
igual; el modo se llama `split`, no split-gb; DOS falsos del propio andamio
por `pipefail` (un `grep -q` que corta el pipe y un `VAR=$(cmd|tail)` que
muere antes del diagnóstico) parieron el rito «el diagnóstico debe
sobrevivir al fallo que diagnostica»; y un `exit` en shell interactivo que
mató una terminal parió el del cat-heredoc + `bash fichero`. Build medido:
cvc5 1.3.2 (git 86cecd8) + CoCoA 0.99800 + CaDiCaL 2.1.3 + Poly 0.2.0. La
72 queda CERRADA CON ACTA en el BACKLOG; doc §2 y la Capa 3 del mapa,
marcadas.

## 191. El censo viaja: credit_climb CENSADO Y COMPUERTA — el hermano por derivación anclada

El resto de la 71 arranca por donde el traspaso mandaba. La quinta pieza de
la estirpe, `doc/fv/interprete_credit_climb.py` (425 líneas), no se escribió:
se DERIVÓ del intérprete de mint con siete cortes anclados count==1 — antes
de derivar se midió que las 12 anclas del constructor de periódicas son
IDÉNTICAS en ambos circuitos: la lógica compartida viaja intacta, drift
cero. Medido sano: **12 clases · 468 celdas-clase · 83 libres declaradas ·
0 sin dueño · 0 rancias · mutantes internos 2/2** (C_HORNER→6 huérfanas en
col 34; C_SALT_IN_A→4, el cerrojo #2 de §117). Las 8 líneas CELDAS_LIBRES:
salt testigo (clase *), bit de camino (sin acct_link), descansos del
acumulador (sin cont_s), limbos altos del primer merge en AMBOS carriles
(§92.2), y carriles muertos tras la raíz — con los digest 4..7/16..19
acreditados por las aserciones de raíz: **§185 trabajando en vivo**.

**El candidato ciego SE REPITE, medido**: borrar `C_SEG_LINK` deja el censo
en delta 0 (Horner co-posee la col 34 en las clases de cierre; lecturas de
P_SEG_LINK 1→0) — sonda interna: `→guardián`. Y el guardián v3 (919 líneas:
`_censo_fv1` compartido, `censo_credit_climb` hermano, `censo_mint_climb`
conserva nombre y byte) lo espera: **tres cazas end-to-end sobre copias** —
candidato borrado → TRIPLE grave ([MUERTA] 3 ranuras + [MUERTA PERIODICA]
[31..33] + [CENSO] el intérprete aborta), exit 1 · C_HORNER borrado → las
dos redes con granos distintos (1 ranura vs 6 celdas-clase) y la sonda viva
girando a `→censo` · selector `cont_s`→`first_s` → redes CIEGAS y el censo
gritando 4 sin dueño exactos, exit 1. Complementariedad bidireccional,
segunda vez, medida. La línea limpia del rito lleva ahora LOS DOS censos.

**Prosa rancia cazada al derivar**: el mensaje del candidato en el
intérprete de mint aún narraba el mundo pre-§189 («huecos sin tumbar el
exit»); corregido en AMBOS intérpretes a la verdad vigente (GRAVES desde
§189, el exit cae solo). Cero contradicciones código↔acta. Quedan de la 71:
`recovery_climb`, `mint`, `send`, `claim` — el patrón, tres veces probado.

## 192. El censo viaja II: recovery_climb — el drift medido se absorbe en un corte

La sexta pieza de la estirpe, `doc/fv/interprete_recovery_climb.py` (425),
derivada del intérprete de mint. El drift, MEDIDO antes de derivar: 11/12
anclas idénticas; la que no — recovery construye `first_row` por asignación
directa, sin bucle — se absorbió en un corte anclado del derivador, con la
máscara {0} intacta. El rito se refina con ello: **drift localizado en un
ancla de forma se absorbe en corte; drift en la lógica compartida veta la
derivación**. Medido sano: **9 clases · 387 celdas-clase · 68 libres
declaradas · 0 sin dueño · 0 rancias · mutantes 2/2** (C_HORNER→5 en col
38; C_SALT_IN_A→4). La cuenta a mano de las cinco formas (36 salt + 7 bit +
3 descansos + 6 limbos + 16 plana) la clavó la máquina. LA COPIA (§93.4) en
la declaración del salt: un testigo viste ambos carriles.

**El candidato ciego, TERCERA confirmación** (hallazgo 6º: es de familia):
delta 0, Horner co-posee la col 38, sonda `→guardián`. Guardián v4 (933
líneas): `censo_recovery_climb` tercero en la tupla de `barrer`; la línea
limpia del rito lleva TRES censos. Tres cazas sobre copias, exit 1 las
tres: candidato borrado → TRIPLE grave ([MUERTA] 1 + [MUERTA PERIODICA]
[31] + [CENSO]) · C_HORNER → ambas redes con grano (1 ranura vs 5
celdas-clase, sonda `→censo`) · selector cambiado → redes CIEGAS, censo 4
sin dueño exactos.

**Diez comentarios rancios del propio circuito, corregidos en la misma
cirugía**: el corrimiento +1 del segundo bit amputado (64.2) numeraba mal
las columnas desde COL_ID_NEW hasta TRACE_WIDTH, y la cabecera decía «fila
271» donde la aritmética resuelve 279. La aritmética manda; ahora la prosa
del .rs la acompaña. Cero contradicciones también dentro del circuito.
Quedan de la 71: `mint`, `send`, `claim`.

## 193. La entrada 73 nace y se ejecuta: el núcleo de la estirpe — una doctrina, N especificaciones

La evaluación que el traspaso ordenó llegó con su dato decisivo: el cuarto
sujeto de la 71 (`circuit_mint`, la familia de PAGOS) mide **10/12 anclas**
— y las dos mudas son de LÓGICA, no de forma: el hash cierra en
`ROW_CUST_ROOT` (dos ascensos: cuentas y custodios), y trae tres familias
de máscara nuevas (`cust_link`, `sel_acct_root`, `sel_cust_root`) más
`P_POW2` y dos bits. El rito veta derivar sobre lógica compartida; la
respuesta es compartirla de verdad: **`doc/fv/censo_nucleo.py`** (397
líneas) — regexes, resolver, clasificación por firma, cosecha conservadora
(§186), censos de transición y aserciones, gramática de CELDAS_LIBRES,
mutantes y candidato, TODO una sola vez — parametrizado por ESPEC
(fichero, anclas, esperados, máscaras, sanidad, P2SEL, rótulo, mutantes,
col de cierre). Los tres intérpretes quedan FINOS: 62/49/49 líneas de pura
especificación (antes 3×425).

**La compuerta reina: paridad byte a byte.** Los tres stdouts completos —
clases, censos, MUTANTES incluidos — capturados como patrón oro ANTES de
tocar nada, y los finos los reproducen BYTE-IDÉNTICOS (diff vacío ×3,
exits 0). El guardián (933, intacto) corre sus tres censos por el camino
nuevo sin enterarse; autotest 3/3; una caza end-to-end de propina
(selector cambiado en credit → censo 4 sin dueño, exit 1). Y una cicatriz
del andamio para el traspaso: el primer generador reventó en un `%s` que
tropezó con el `r % CYCLE_LENGTH` del propio texto del ancla — y el falso
verde resultante (oro contra oro, ficheros sin escribir) lo delató el
stderr y las líneas de escritura AUSENTES: **la paridad se declara tras
ver la pluma escribir, no solo el diff en verde**.

La 71 sigue: `mint` (pagos) será la SÉPTIMA pieza, la primera nacida
directamente como ESPEC sobre el núcleo — con sus familias propias
declaradas, no derivadas.

## 194. La séptima pieza nace ESPEC: circuit_mint (pagos) — dos ascensos, cuatro censos

La primera pieza escrita directamente como especificación sobre el núcleo:
`doc/fv/interprete_mint.py` (94 líneas) declara las familias PROPIAS de
la emisión como delta explícito — hash hasta `ROW_CUST_ROOT` (dos
ascensos), `cust_link` {287, 295, 303, 311}, los selectores de raíz
`sel_acct_root` {279} y `sel_cust_root` {319} nacidos del triple one-hot,
`P_POW2` como columna de VALORES (fuera de P2SEL, como las ARK), y 16
anclas con `let any_link = acct_link + cust_link;` fijada. Para censarla
el núcleo creció dos veces, con paridad byte a byte re-probada ×3 en cada
una: **v2, el alias-SUMA de selectores** (un reclamo por sumando —
conservador; sin él, C_CAP habría reclamado con firma vacía TODAS las
clases: sobre-cobertura, sesgo al verde, §59.2) y **v3, `sin A ni B`** en
la gramática de libres (las firmas compuestas de mint no se nombraban con
un solo selector).

Medido sano: **21 clases · 1029 celdas-clase · 42 sentencias · 343
huérfanas** — y la cuenta a mano (163 económicas + 180 de carriles) la
clavó la máquina. **19 declaraciones → 0 sin dueño · 0 rancias a la
primera.** Mutantes 2/2 (C_HORNER→10 en col 44; C_SALT_IN_A→4). **El
candidato ciego, CUARTA confirmación** — ahora fuera de la familia
*_climb: la co-posesión Horner/seg_link es del PATRÓN de segmentos, no de
la estirpe. Guardián v5 (947): `censo_mint_pagos` cuarto en `barrer`; la
línea limpia lleva CUATRO censos; autotest 3/3. Tres cazas exit 1:
candidato → TRIPLE grave ([MUERTA] 8 ranuras + [MUERTA PERIODICA] 8 cols
+ [CENSO]) · Horner → ambas redes (1 vs 10, sonda →censo) · selector →
redes ciegas, censo lo para. Tres rancios del .rs corregidos en la misma
cirugía: la cabecera decía «45 columnas» (son 49), la tabla de ciclos
arrastraba el corrimiento pre-salt, y un `#[cfg(test)]` duplicado.

Quedan de la 71: `send`, `claim` — con el molde ESPEC ya caliente.

## 195. FV-1: el ENVÍO nace ESPEC — octava pieza, primera fuera de la traza 512

La octava pieza del censo FV-1, y la primera FUERA de la traza 512:
`circuit_send`, dos carriles reales sobre 1024×56. Nace directamente como
ESPEC (`doc/fv/interprete_send.py`, 129) sobre el núcleo: 21 anclas con
conteo exacto, 20 constantes ESPERADAS, 16 selectores en P2SEL, 42
aserciones con clase. El censo mide 23 clases · 1288 celdas-clase · 56
sentencias — y aquí VUELVE la clase plana: la tubería acaba en la fila
815 y deja 208 filas de holgura que ninguna pieza anterior tenía.

Send exigió el núcleo v4 (409→415), dos formas nuevas y un corte por
cada una: arrays con suma de literal (`transport` lleva `COL_KEY +
1..3`; los elementos ahora se EVALÚAN, no se buscan por nombre) y la
suma INLINE de selectores (`(frozen_entry + frozen_link)` sin `let`):
un reclamo POR SUMANDO, con los sumandos fuera del conjunto estricto.
Paridad byte a byte ×4 + autotest contra el patrón oro, en el espejo y
en este árbol — la pluma vista escribir en ambos.

258 huérfanas medidas y leídas UNA A UNA contra el circuito; 13
declaraciones (33 líneas en el .rs, 2449→2483); 0 sin dueño · 0
rancias. Tres familias con historia: el salt testigo (52..55) libre en
TODAS las clases —solo se lee, nunca se ata—; el carril B libre en la
fase de congelados —la no-pertenencia sube SOLO por A y B jamás se
asierta ahí—; y los limbos altos del nonce (9..11/21..23 en
link_leaf): el censo REDESCUBRE por su cuenta la cuestión previa de la
spec §4 y la deja declarada citando al test que mide la defensa aguas
abajo. Una lección de gramática cazada midiendo, no leyendo: el `sin`
va DENTRO de `clase` (RE_LOCUS) — 63 huérfanas fantasma hasta dar con
la forma canónica.

Guardián v6 (947→961, +14): QUINTO censo en `barrer`, línea limpia con
los cinco, autotest 3/3. Cazas ×3 con restauración por sha: candidato →
TRIPLE grave ([MUERTA] 5 + [MUERTA PERIODICA] 5 + [CENSO]; NS=5, no
8) · Horner → ambas redes (sonda 1 vs censo 10) · selector → redes
ciegas y el censo grita 8 EXACTOS. El candidato ciego suma su QUINTA
confirmación: la co-posesión Horner/seg_link es del PATRÓN — ya ni la
longitud de traza es de la estirpe. Pista rancia anotada:
`with_capacity(29)` contra 42 aserciones reales. Suites canónicas en la
forma del §194 (por paquete, release, compuertas literales): stark 297
· zk-ssl 242 · 0 failed · warnings 0·0.

**Estado 71**: cinco piezas censadas —mint_climb, credit, recovery,
mint-pagos y send, la primera fuera de la traza 512— con el censo como
COMPUERTA del guardián (v6, 961) y el núcleo en v4. Queda `claim`. Canon sin cambio (297 · 242 · 40 · 28).

## 196. FV-1: la RECLAMACIÓN nace ESPEC y la 71 SE CIERRA — novena pieza, seis censos

La novena pieza, y con ella la entrada 71 completa su lista: la gemela
invertida de send. `doc/fv/interprete_claim.py` (82 líneas) censa
`circuit_claim` — 1024×55, sin `COL_LIMIT`, 4 segmentos Horner que
acaban en la fila 255 — con 22 anclas de conteo exacto, incluida la que
fija el cierre de §39 en el propio código: el compromiso interno ata la
identidad a `COL_ACC_ID`, la cuenta que COBRA, no a `COL_R_ID` (muerto
tras §39.1; su constancia sigue impuesta por el transporte). Las dos
inversiones están en el código y el censo las lee sin inmutarse: el
saldo SUBE y el pendiente SALE (carril A arrastra el compromiso, B
entra a cero) — la prosa vieja del propio .rs aún narraba la entrada;
la aritmética manda.

Antes de censar, el rito cazó un import fatal EN SECO: importar un
ESPEC lo EJECUTA (`sys.exit`), así que compartir máscaras entre
gemelas importándolas era un suicidio. La respuesta de §193, otra vez:
**núcleo v5** (461) — `mascaras_gemelas` y `P2SEL_GEMELAS` viven UNA
vez en el núcleo; send adelgaza 129→88 y su salida se re-probó
BYTE-IDÉNTICA. Paridad ×5 contra patrón oro, espejo y árbol.

Medido sano: **21 clases · 1155 celdas-clase · 55 sentencias · 235
huérfanas → 12 declaraciones (29 líneas en el .rs, 2179→2208) → 0 sin
dueño · 0 rancias, limpio A LA PRIMERA** — la lección de gramática de
§195 ya estaba pagada. Los segmentos cortos estrenan clases PURAS que
send no tenía: `sel_root`, `frozen_entry+sel_pk_done` y una
`frozen_link` unificada de 31. Los limbos altos del nonce quedan
declarados citando a los tests-decisores de send (aquí no viven): misma
siembra, misma defensa aguas abajo. Pista rancia anotada, tercera
aparición: `with_capacity(29)` contra 41 aserciones reales.

Guardián v7 (961→975, +14): SEXTO censo, línea limpia con los seis,
autotest 3/3. Cazas ×3 con restauración por sha: candidato → TRIPLE
grave ([MUERTA] 4 + [MUERTA PERIODICA] 4 + [CENSO] — la escalera NS
8/5/4 por circuito) · Horner → ambas redes (sonda 1 vs censo 7) ·
selector → censo 5 EXACTOS. **El candidato ciego, SEXTA confirmación**:
la co-posesión Horner/seg_link es del PATRÓN de segmentos — probada ya
en dos longitudes de traza, dos topologías de carril y seis circuitos.
Suites canónicas: stark 297 · zk-ssl 242 · 0 failed · warnings 0·0.

**Estado 71: SE CIERRA.** Seis piezas censadas —mint_climb, credit,
recovery, mint-pagos, send y claim— con el censo como COMPUERTA del
guardián y la honestidad de serie: referenciada ≠ determinada (doc §1).
Lo que queda fuera ya está registrado como decisión, no como deuda:
extender a los 28 es elección (el candidato ×6 dice que lo que importa
es el patrón), y `dual_climb` sigue siendo el hueco conocido de índices
crudos. El checkbox baja. Canon sin cambio (297 · 242 · 40 · 28).

## 197. Ecosistema RPC (Fase 0): el pago en dos fases sale por el cable — y la clave no viaja

Material de OTRO taller, AUDITADO antes de aplicado. Un paquete generado
en otra sesion (zip sha `bb2ff940…`) trajo cuatro crates —`zk-ssl-wire`
(584: el formato de cable, hex canonico validado), `zk-ssl-cli`
(301→327+cinco modulos: sandbox/trazador sobre la capa REAL),
`zk-ssl-node` (417: JSON-RPC 2.0 de referencia, axum) y `zk-ssl-sdk`
(275+38: el lado del titular)— mas `spec/RPC.md` (zkssl/0.1) y tres
documentos. Antes de tocar el arbol: las 2.244 lineas de Rust y los
cuatro docs LEIDOS ENTEROS, ~40 firmas verificadas contra el codigo
sellado (todas casaron; dos «ausencias» fueron greps mios en el fichero
equivocado — ausencia-en-grep no es ausencia), y CERO copias de ficheros
sellados dentro del paquete.

Cinco parches minimos con pre/post por sha, ninguno de logica: members
del workspace; feature `sandbox`; visibilidad de `tests_support` tras
esa feature; `pub fn open_with_id` — la puerta de `zkssl_openAccount`,
cuya doctrina ya vivia en el propio metodo («la clave no se almacena en
ningun sitio», §93.4: con esto, tampoco viaja); y la copia. `cargo
check` de los cuatro: VERDE A LA PRIMERA (22 s) — el paquete se escribio
contra las firmas reales sin poder compilar, y compilo.

El humo destapo UN bug real, arreglado por la regla de oro (solo crates
nuevos): `simulate` asumia indices secuenciales, pero el arbol de
cuentas es DISPERSO — abrir devuelve la POSICION DERIVADA de la hoja
(#296740276, #4052836150…). El fix mapea logico→real en la corrida.

**Fase 0 del roadmap: CUMPLIDA por dos vias.** CLI: fund×2 + send +
claim con pruebas reales — saldos 750000/1250000, en transito 0, cadena
integra (6 entradas). SDK contra el nodo VIVO con claves ALEATORIAS (la
via de produccion): alice 750000 · bob 250000 · `verifyChain` 7
entradas `ok:true`. La promesa «la clave de gasto no viaja» quedo
verificada en el CODIGO (unica aparicion de `spend_key`: dentro de
`prove_send/prove_claim`, en local) y en EJECUCION.

Medidas que el papel no tenia: pruebas reales de **54–66 KB** segun
circuito (la spec decia ~36,7: corregida); **determinismo POR
OPERACION** cruzado en dos superficies — mismas raices y cadena en
CLI↔RPC con la misma semilla (`f3fca118…`/`1a9be8eb…`): el cimiento de
los vectores de conformidad; el `epochHead` virgen con seq 0, cadena a
ceros y la raiz del arbol VACIO por triplicado (`cef70622…`); y
serde_json ordenando claves en ALFABETICO (BTreeMap) — dato para
implementadores y para cualquier gate. Guardian tras los parches:
barrer con los SEIS censos, circuitos NI ROZADOS; suites 297/242,
warnings 0·0. Las deudas que el RPC hereda quedan DECLARADAS, no
escondidas, y viven en la **nota 74, que esta entrada ABRE** (Fase 1:
OpenRPC · vectores por version · proceso RFC · keystore). Canon sin cambio (297 · 242 · 40 · 28).

## 198. Fase 1a de la 74: OpenRPC desde el wire y los vectores de conformidad — el determinismo, de hallazgo a compuerta

Dos piezas para que exista una SEGUNDA implementacion, y las dos con la
misma idea: una sola fuente que se auto-verifica.

**OpenRPC (`spec/openrpc.json`, 412, GENERADO).** La tabla de los 17
metodos vive en `zk-ssl-wire/src/openrpc.rs` (135), junto a los DTOs que
describe — no en un documento aparte que envejezca. `gen_openrpc` la
vuelca; dos tests la vigilan (17, unicos, en el orden de `spec/RPC.md`);
y el BLOQUE-198 exige que REGENERAR reproduzca el fichero BYTE A BYTE:
el json no puede divergir de la tabla sin gritar. v0 deliberadamente
conciso — esquemas por referencia (Q/DATA/Digest con el patron hex
canonico); el contraste campo a campo no es suyo: es de los vectores.

**Vectores de conformidad (`spec/vectors/zkssl-0.1.json`, 64).** El
escenario canonico —open+fund(0xA11CE,+1000000)×2 · send 250000 (salt
7) · claim— reducido a hechos: por operacion, raiz vieja y nueva,
digest de la prueba y digest de cadena; al final, cabeza de epoca y
suministro. Todo en el hex de `store::digest_to_bytes`: el mismo byte a
byte que persiste la capa. `conformance --check` re-ejecuta y compara
campo a campo; en la maquina del sello paso DOS veces seguidas —
**el determinismo por operacion (§197) deja de ser hallazgo y pasa a
COMPUERTA**, re-probada en cada BLOQUE futuro. Y una tercera
confirmacion cruzada, gratis: el `epoch_digest` del escenario
(`0x58a324cc…`) es EXACTAMENTE el del `simulate` de la Fase 0 — dos
caminos de codigo, un estado final.

Proceso RFC en `spec/rfc/` (PROCESO 24 + plantilla 29), con la regla
que protege a los vectores: si el cable cambia, la version sube y los
vectores viejos se CONSERVAN bajo la suya — jamas se reescriben.

Ejecucion del F1: ~300 lineas de Rust nuevas, `cargo check` y tests
VERDES A LA PRIMERA; cinco parches anclados (serde_json al wire, el
modulo, el subcomando) — todo en crates nuevos y `spec/`; la capa y los
circuitos NI ROZADOS (huellas en el BLOQUE). De la nota 74 queda:
keystore cifrado, y los RFC de fondo cuando toquen. Canon sin cambio (297 · 242 · 40 · 28).

## 199. El keystore — y la 74 SE CIERRA: de implementacion a protocolo, con el wallet dormido bajo la ley del ledger

La clave de gasto ya no viajaba (§197); ahora tampoco DUERME desnuda.
`zk-ssl-sdk/src/keystore.rs` (217) guarda el wallet cifrado con la MISMA
construccion de reposo que el ledger — `zk_ssl::crypto`: XChaCha20-
Poly1305, nonce de 24 bytes aleatorio por escritura, clave derivada de
la contrasena con SHA-256 sobre un DOMINIO — pero con dominio PROPIO
(`ZK-SSL-keystore-v1`): **ledger y wallet nunca comparten clave
simetrica aunque compartan contrasena**, y eso no es una intencion sino
un test — `la_clave_del_ledger_no_abre_el_keystore` construye la
LedgerKey con la misma contrasena y EXIGE que el descifrado falle. Una
sola ley de reposo en el proyecto, dominios separados probados; escribir
criptografia propia aqui habria sido el error que el propio Cargo.toml
de la capa advierte.

El fichero (`zkssl-keystore/1`, JSON de la casa) lleva EN CLARO el
`public_id` — publico por diseno — y cifrados los 32 bytes canonicos de
`store::digest_to_bytes`. Al cargar, el binding declarado==derivado
rechaza el fichero cambiado de sitio o editado; el cifrado autenticado
rechaza la manipulacion byte a byte; en Unix el fichero nace 0600. Cinco
tests (ida y vuelta de identificadores, contrasena mala, manipulacion,
public_id ajeno, DOMINIOS) mas el doctest del SDK y el ejercicio vivo
(`examples/keystore.rs`, 25): guardar, cargar identico, y que lo malo
FALLE — todo VERDE A LA PRIMERA en la maquina del sello. Cero deps
nuevas de verdad: chacha20poly1305 y sha2 entran al SDK en las MISMAS
versiones que ya vivian en el lock. La advertencia KDF de la capa aplica
INTEGRA y viaja escrita en el modulo; endurecer a Argon2/scrypt tiene
cauce — el proceso RFC de §198 — no parche silencioso.

**Con esto la nota 74 baja a [x]**: spec (`zkssl/0.1`) + OpenRPC
generado + vectores de conformidad como compuerta + proceso RFC + nodo
de referencia + SDK con el wallet en reposo. Lo que era una
implementacion es un PROTOCOLO con contrato publico y segunda-
implementacion posible. Las deudas declaradas de la nota (aviso fuera
de banda §21, nodo unico, --dev) siguen declaradas donde estaban: el
cierre no las esconde, las deja escritas con su cauce. Canon sin cambio (297 · 242 · 40 · 28).

## 69. Qué NO demuestra este documento

⚠️ **Esta seccion se queda la ultima a proposito, aunque §70 y §71 la
precedan en el fichero.** Es el cierre del documento, no un hallazgo mas:
ponerla en medio haria que el texto concluyera y siguiera hablando. La
convencion de no renumerar se mantiene; lo que se declara es que el orden
de lectura no es el numerico, igual que en `BACKLOG.md`.

Que el sistema sea seguro. Demuestra que **el autor ha buscado sus
propios fallos de forma sistemática y ha encontrado algunos**, incluidos
dos al escribir estas páginas.

Es exactamente por eso que hace falta que lo mire alguien más.
