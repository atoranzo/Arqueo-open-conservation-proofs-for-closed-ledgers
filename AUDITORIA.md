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

**Ninguna está implementada.**

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

### 6.1 Matriz por operación

| Operación | El operador | La contraparte | Un tercero |
|---|---|---|---|
| **Transferencia** | Todo | **El saldo del receptor** ⚠️ §4 | Nada |
| Emisión | Todo | Custodios: **el importe** | **El importe** |
| Destrucción | Todo | — | **El importe** |
| Auditoría | Lo revelado | Supervisor: **la identidad de la cuenta** | Nada |
| Recuperación | Todo | Custodios: la identidad nueva | Nada |
| Congelación | Todo | Custodios: **qué cuenta** | Nada |
| Gasto sin conexión | — | Comercio: el importe | Nada |

### 6.2 Lo implícito, ahora declarado

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

### 6.3 Lo que el repaso confirmó que SÍ está cerrado

| Propiedad | Cómo |
|---|---|
| La emisión no revela la cuenta acreditada | Solo raíces, no identidades |
| La destrucción no revela la cuenta | Ídem |
| El nullifier no se deriva de la identidad pública | Test: `nullifier_is_not_derivable_from_public_id` |
| Una transferencia no revela identidades a terceros | Solo raíces y nullifier |

### 6.4 La pregunta que queda para un auditor

**¿Hay más datos que se entregan por necesidad técnica y que nadie
examinó?**

La fuga de §4 existía porque un comentario la justificaba —*"información de
estado, no secretos"*— en vez de examinarla. **Ese patrón puede repetirse
en sitios que este repaso no ha mirado**: la API de instantáneas, los
materiales de auditoría, o cualquier cosa que se añada después.

---

## 6. Donde el autor tiene MENOS confianza

Esta es la sección más útil del documento.

### 6.1 `open_account` no exige autorización — **mitigado a medias**

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

### 6.2 La congelación no tiene justificación ni caducidad

**Implementada** con imposición en circuito: la prueba de liquidación
acredita que el emisor no está en el árbol de congelados.

**Lo que queda abierto:**

- El circuito demuestra que dos custodios la autorizaron, **no que
  tuvieran razón**. No hay orden judicial ni motivo registrado.
- **No hay caducidad**: una congelación dura hasta que alguien la levante.
- Una cuenta congelada **sigue pudiendo recibir**. Es deliberado —lo
  contrario dejaría fondos en el limbo— pero merece que un auditor valore
  si encaja con el caso de uso.

### 6.3 Los grados de restricción

**Cinco veces** durante el desarrollo winterfell rechazó un grado mal
declarado. Cada vez se corrigió. La exactitud que exige winterfell hace
improbable que un grado incorrecto pase inadvertido, **pero la
concentración de errores en este punto sugiere revisarlo con atención**.

Especial cuidado con:
- Restricciones que multiplican **dos columnas periódicas** (`C_ACC`).
- Columnas periódicas cuyo periodo real difiere del declarado
  (`circuit_mint`: 8 segmentos × 64 filas llenan la traza y la vuelven
  periódica de periodo 64).

### 6.4 El patrón lockstep

`C_SIBLING` impone que los dos carriles usen el mismo hermano. El
argumento es que eso basta para atar ambas subidas a la misma posición
del árbol.

Está verificado con un test discriminante, **pero el argumento general no
ha sido revisado por nadie más**. Es el hallazgo más original del
proyecto y merece escrutinio.

### 6.5 Los tests negativos

**Tres veces** un test negativo resultó no discriminar: fallaba por una
restricción distinta de la que pretendía probar. Se corrigieron
construyendo testigos internamente coherentes.

**Puede quedar alguno más.** Un auditor debería comprobar, para cada test
negativo, que el testigo corrupto es válido en todo lo demás.

### 6.6 El bloqueo de directorio de `sled` tras cerrar — **hallazgo nuevo**

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

### 6.7 El techo de 63 bits

Las comprobaciones de rango fuerzan el bit más significativo a cero, así
que **ningún valor puede superar 2^63 − 1**.

La capa **no valida** que `max_supply` esté por debajo de ese techo. Con
un tope mayor, las emisiones fallarían con un error confuso en vez de
rechazarse al configurar.

No es una fuga de solidez —los valores fuera de rango se rechazan— pero
sí un fallo de usabilidad que puede confundir un diagnóstico.

### 6.8 El formato de instantánea se queda atrás al añadir estado

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

### 6.9 Colisiones en el árbol de nullifiers

La posición sale de los bits bajos del nullifier. Dos nullifiers pueden
colisionar, y el segundo **no podría gastarse**.

El autor lo clasifica como **denegación de servicio, no ruptura de
solidez**. Ese razonamiento merece verificación independiente: si fuera
incorrecto, sería grave.

---

## 7. Por dónde empezaría el autor si tuviera que romperlo

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

## 8. Limitaciones ya documentadas

No hacen falta descubrirlas; están en `README.md`:

- El operador del nodo es un intermediario de confianza.
- No hay red, consenso, réplicas ni cifrado en reposo.
- La generación de la prueba puede hacerse en el cliente (`client`), pero
  delegarla a un **tercero** exigiría verificar una firma en circuito.
- La resolución IBAN → cuenta está fuera de la prueba.
- El conjunto de gobernanza es inmutable.
- Las cifras de rendimiento son de una sola ejecución.

---

## 9. Cómo reproducir

```bash
cargo test -p zk-ssl --release              # la capa, 65 tests
cargo test -p stark-experiment --release    # los seis circuitos
cargo test -p zk-ssl --release metrics -- --nocapture
```

**Los tests de circuito conviene ejecutarlos también en debug**: winterfell
valida las restricciones al generar solo en ese modo, y da el índice y la
fila exactos del fallo.

---

## 10. Qué NO demuestra este documento

Que el sistema sea seguro. Demuestra que **el autor ha buscado sus
propios fallos de forma sistemática y ha encontrado algunos**, incluidos
dos al escribir estas páginas.

Es exactamente por eso que hace falta que lo mire alguien más.
