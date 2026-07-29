# ZK-Sovereign Settlement Layer (ZK-SSL)

Capa de liquidación con privacidad criptográfica y cumplimiento
demostrable, **sin ninguna ceremonia de confianza**.

```rust
let mut layer = SovereignLayer::open("./ledger", issuer_key, limite, tope)?;

let alice = layer.open_account(sk_alice);          // saldo CERO
let recibo = layer.mint(issuer_key, alice, 1_000_000)?;
layer.apply_mint(&recibo, alice)?;                 // EXIGE clave del emisor

let envio = layer.send(sk_alice, alice, &estado, id_bob, aleatorio, 250_000)?;
layer.apply_send(&envio, alice, &estado, 250_000)?;   // el dinero sale
let cobro = layer.claim(sk_bob, bob, &estado_bob, &envio.notice)?;
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;  // el receptor cobra
```

Con **persistencia**: el ledger sobrevive al reinicio, y un estado
corrupto se detecta antes de operar sobre él.

```rust
let mut layer = SovereignLayer::open("./ledger", issuer_key, limite)?;
```

Con **revelación selectiva**: un supervisor puede auditar sin que la
privacidad se rompa para nadie.

```rust
let d = layer.audit(sk, cuenta, 900_000, 1_100_000)?;  // "estoy entre X e Y"
verify_audit(&d)?;                                      // el supervisor, SIN la capa
```

Con el **ciclo monetario completo**: el dinero puede crearse, moverse y
retirarse, y la invariante global se mantiene en cada paso.

`crates/zk-ssl` — **172 tests**. Material para auditoría externa en
[`AUDITORIA.md`](./AUDITORIA.md), todos en release. El backend STARK
añade 5 circuitos verificados por separado.

## ⚠️ Lo primero: el operador del nodo ES un intermediario de confianza

Este proyecto se construyó sobre el principio de **eliminar la
dependencia de intermediarios de confianza centralizados**. Se eliminó
uno —los participantes de la ceremonia de setup, que en Groth16 o
PLONK-KZG podrían coludir y crear dinero sin dejar rastro— y esa es una
propiedad real.

**Pero queda otro, y es mayor.** Esta capa es un **nodo único**. Quien lo
opera:

- **Ve todos los saldos.** La privacidad es frente a terceros que solo
  ven pruebas, no frente a quien mantiene el estado.
- **Ordena las operaciones.** Decide qué entra y en qué orden.
- **Puede censurar.** Nada obliga a procesar la operación de nadie.
- **Es un punto único de fallo.** Incautarlo, apagarlo o corromperlo es
  incautar, apagar o corromper el sistema entero.

Es literalmente el intermediario de confianza centralizado que el
principio fundador señala. Está aquí en la cabecera, y no en una lista de
limitaciones, porque **enterrarlo sería vender una propiedad mientras se
esconde su contraria**.

### Qué está demostrado y qué no

| Afirmación | ¿Demostrada matemáticamente? |
|---|---|
| Esta transferencia conserva el dinero | ✅ |
| El saldo de esta cuenta es X | ✅ |
| No se ha creado dinero fuera del suministro | ✅ |
| Nadie gasta sin ser el titular | ✅ |
| Nadie gasta dos veces | ✅ |
| **Este es el estado actual del sistema** | ❌ **Confianza en el operador** |
| **Estas son todas las operaciones que hubo** | ❌ **Nada impide omitir** |

**Las transiciones de estado están demostradas. El estado y la
completitud del historial, no.** Cerrar esa brecha requiere consenso
distribuido — un problema de sistemas distribuidos, no de criptografía, y
trabajo pendiente.

### Qué es esto entonces, con precisión

**Una demostración de que las propiedades criptográficas de una
liquidación soberana son construibles y medibles**: privacidad,
cumplimiento demostrable, conservación del dinero, auditoría selectiva y
ausencia de ceremonia de confianza, todo con tests que buscan romperlo.

**No es** una capa de liquidación descentralizada. Sin consenso
distribuido, los principios de neutralidad y soberanía no se cumplen
todavía.

---

## Por qué STARK, y por qué eso NO era obvio

El proyecto implementó el mismo circuito en **cinco paradigmas** (Groth16,
Halo2/IPA, STARK/FRI, PLONK/KZG y Nova) y midió sus trade-offs. La
elección se deduce del principio de soberanía:

> **Groth16 y PLONK-KZG exigen una ceremonia de confianza.** Si sus
> participantes coluden, pueden falsificar pruebas y **crear dinero de la
> nada** sin que nadie lo detecte jamás.

Para una infraestructura soberana eso es una dependencia externa
permanente e inauditable. Sin ceremonia quedan Halo2/IPA y STARK/FRI; de
los dos, STARK gana en todo salvo el tamaño de prueba y es el único con
resistencia cuántica.

**Se descartó Groth16 pese a ser el más rápido y el de pruebas más
pequeñas** (192 bytes frente a 62 KB). La coherencia con el principio pesó
más que el rendimiento.

### La diferencia, medida

| | Groth16 | **ZK-SSL (STARK)** |
|---|---|---|
| Arranque de la capa | 2,6 s generando claves | **1,12 ms** |
| Ceremonias necesarias | Una **por circuito** | **Ninguna** |
| Prueba de emisión | 192 B | 53.164 B |
| Prueba de liquidación | 192 B | 61.966 B |
| Resistencia cuántica | No | **Sí** |

Que el constructor sea `new(issuer_key, limite)` en vez de devolver
`Result` tras generar claves **es la propiedad hecha código**: no hay nada
que generar ni ningún secreto que destruir.

## Escrituras atómicas: se acabó la intervención manual

**El problema**: una transferencia hacía **cuatro llamadas con nueve
escrituras** —dos cuentas, un nullifier y seis valores de metadatos—. Si
el proceso moría en medio, quedaban aplicadas unas sí y otras no, y el
arranque siguiente detectaba la inconsistencia y **se detenía hasta
intervención manual**.

**La corrección**: agruparlas en un solo lote atómico.

### Por qué NO se escribió un log de escritura anticipada

`sled` ya garantiza que un `Batch` se aplica entero o no se aplica.
Construir un WAL propio encima habría sido reimplementar —con más
superficie de fallo— algo que el motor de almacenamiento ya hace.

**El problema nunca fue la falta de un WAL**: era que hacíamos nueve
escrituras sueltas donde debía haber una operación. Diagnosticarlo bien
ahorró la mitad del trabajo y todo el riesgo.

### La garantía, con precisión

| Momento del fallo | Estado resultante |
|---|---|
| Antes de aplicar el lote | El anterior, coherente |
| Entre aplicar y sincronizar | Uno de los dos, **nunca a medias** |
| Después de sincronizar | El nuevo, coherente |

En los tres casos el ledger queda coherente. Lo que se puede perder en el
caso intermedio es **durabilidad** —la operación quizá no se guardó— pero
no **integridad**.

Esa distinción es la que importa: **perder una operación es recuperable**
(se vuelve a enviar); **un estado a medias no lo es**.

### ⚠️ Lo que los tests NO comprueban

**Matar el proceso a mitad de escritura.** Eso requiere tests a nivel de
proceso, no unitarios. Lo que sí se verifica es que la coherencia se
mantiene tras cada operación, y la verificación de integridad al arrancar
detectaría cualquier divergencia.

## Instantáneas: un disco perdido ya no es el ledger perdido

```rust
let info = layer.export_snapshot("./copia.bin")?;
let restaurada = SovereignLayer::import_snapshot("./copia.bin")?;
```

### Tres decisiones que la hacen útil

**Formato binario propio, no el del motor.** Una copia de archivo debe
poder leerse dentro de diez años sin depender de la versión de `sled` que
la escribió.

**Orden determinista.** Dos instantáneas del mismo estado son **byte a
byte idénticas**. Sin eso, compararlas para detectar divergencias entre
nodos sería inútil.

**La importación verifica.** Reconstruye los árboles y compara las raíces
con las declaradas. Sin eso, restaurar una copia manipulada haría que el
nodo generase **pruebas válidas sobre un ledger que no es el real** — y
las pruebas verificarían perfectamente.

### ⚠️ Lo que NO es

- **No hay replicación en vivo.** Es una copia puntual; los cambios
  posteriores no se propagan. Replicar en caliente exige red y
  coordinación.
- **No hay copias incrementales.**
- **Las instantáneas van cifradas** con la misma clave que el ledger, si la
  hay. **Sin clave van en claro**, y entonces quien tenga el fichero ve
  todos los saldos.

## Cifrado en reposo

```rust
let key = crypto::LedgerKey::from_passphrase("...");
let layer = SovereignLayer::open_encrypted(path, ..., Some(key))?;
```

**XChaCha20-Poly1305** de RustCrypto, cableado a todas las escrituras y
lecturas del ledger. Escribir criptografía propia aquí habría sido un
error grave.

**Nonce de 24 bytes aleatorio por escritura.** Con 12 habría que llevar un
contador, y un reinicio mal gestionado reutilizaría uno — lo que rompe la
confidencialidad.

**Autenticado**: una manipulación se detecta al descifrar. Sin eso, quien
tuviera el disco podría alterar saldos **sin conocer la contraseña**.

### El test que lo comprueba de verdad

`balances_are_not_readable_on_disk` busca el saldo en claro **byte a byte
entre todos los ficheros**. Los otros dos tests —que la contraseña
correcta abre y la incorrecta no— pasarían incluso con un XOR trivial.

Y hay un tercero, `without_encryption_the_balance_is_readable`, que
comprueba que **sin cifrado el saldo SÍ aparece**. Si no apareciera, la
búsqueda del primero no comprobaría nada y pasaría siempre.

> Esa disciplina evitó un error real: la primera versión del test fallaba
> con un valor que superaba el tope de emisión, así que el saldo nunca se
> creaba. Parecía una fuga y no lo era.

### ⚠️ El alcance, y es más estrecho de lo que parece

**Protege contra**: robo del disco, de una copia, o de una instantánea.

**NO protege contra**: el operador del nodo —que ve los saldos en
memoria—, ni contra alguien con acceso al proceso en marcha.

Leerlo como "los saldos son privados" sería un error. Eso solo lo corrige
la descentralización.

### Y su consecuencia operativa

La clave la aporta el operador al arrancar; guardarla junto a los datos no
protegería nada. Eso significa que **el nodo no puede reiniciar solo**:
alguien tiene que introducir la contraseña.

Además, la derivación usa SHA-256, que **no es una función de derivación
de contraseñas**: no tiene coste ajustable, así que una contraseña débil
es vulnerable a fuerza bruta. Un despliegue real necesitaría Argon2 o
scrypt.

## La clave de gasto ya no llega al nodo

**El problema, que estaba sin documentar**: `layer.transfer(sender_key, ...)`
recibía la clave de gasto. Es decir, **para transferir había que
entregársela a quien opera el nodo**, y con ella puede vaciar la cuenta
cuando quiera.

No era una limitación de escala: era que el sistema exigía **confiar tu
dinero al operador**, precisamente el intermediario que el proyecto dice
eliminar.

### El protocolo

```rust
// FASE 1 — el pagador. La capa no ve su clave.
let m = layer.send_materials(alice, id_de_bob, importe, aleatorio)?;
let envio = client::prove_send(&m, key, proof_options())?;   // LOCAL
layer.apply_send(&envio, alice, &estado_alice, importe)?;

// FASE 2 — el receptor. Tampoco ve la suya.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, key_bob, proof_options())?;  // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;
```

⚠️ **Este protocolo sustituye al de un paso, retirado.** Aquella vía
—`transfer_materials` / `prove_transfer`— entregaba al pagador **una vista
completa del receptor, con su saldo**, porque actualizaba las dos hojas a la
vez. Ver `AUDITORIA.md` §29.

**Los materiales del envío llevan solo el identificador del receptor**, y el
tipo lo impone: no hay campo por donde el saldo pudiera entrar.

⚠️ **Asimetría del cobro**: `claim_materials` recibe el aviso, no lo entrega.
**La capa no sabe qué pendiente es de quién** —esa es la privacidad del
diseño— así que no podría decírselo al receptor. Cómo le llega el aviso es la
pieza que ISO 20022 no transporta.

**`prove_send` y `prove_claim` son funciones libres, no métodos de la capa.** Es
deliberado: la capa **no puede** llamarla porque no tiene la clave. Si
fuera un método, la API sugeriría lo contrario y alguien acabaría
pasándosela.

El nullifier sí viaja a la capa, pero **no revela nada nuevo**: es
público y aparecería igualmente al aplicar la liquidación. Lo que no
viaja es la clave que lo genera.

### La propiedad que lo hace seguro

`materials_alone_are_not_enough_to_spend`: un atacante con **todos los
materiales** —caminos de Merkle, saldos, nonces— pero sin la clave **no
puede generar la prueba**. Por eso los materiales pueden viajar por un
canal cualquiera.

### ⚠️ Lo que esto NO resuelve

**El operador sigue viendo los saldos.** La capa mantiene el estado, así
que los conoce. Esto elimina que vea **claves**, no que vea **datos**.

**Generar la prueba cuesta ~600 ms y memoria.** Un cliente ligero que
quiera delegar el cómputo a un tercero necesitaría que ese tercero
pudiera probar **sin** la clave, lo que exige verificar una firma dentro
del circuito (Winternitz, ~8.000 filas más). Eso es una **optimización
para clientes ligeros**, no una corrección de seguridad: la custodia
queda resuelta aquí.

### Una corrección de análisis

Este problema se pospuso durante el proyecto con el argumento de que
"presuponía una arquitectura descentralizada". **Era falso**: la parte
grave —que la clave llegue al nodo— se resuelve separando la API, sin
criptografía nueva. Lo caro es solo la delegación a terceros, que es
opcional.

## Registro de transiciones: el operador no puede reescribir el historial

### ⚠️ Esto NO es descentralización

El operador de un nodo único tiene tres poderes. Este registro cierra uno:

| Poder | ¿Cerrado? |
|---|---|
| Ve todos los saldos | **No** |
| Ordena las operaciones y puede censurar | **No** |
| **Bifurcar o reescribir el historial** | **Sí** |

Los dos primeros exigen consenso distribuido, que es otra disciplina y no
está abordado. Lo que se cierra aquí tiene nombre propio: **no repudio del
historial**.

### Cómo

Cada operación deja una entrada encadenada:

```text
resumen = H(numero, tipo, raiz_antigua, raiz_nueva, H(prueba), resumen_anterior)
```

**Publicar la cabeza basta.** 32 bytes comprometen todo el historial: dos
copias con la misma cabeza tienen la misma historia, y una reescritura
posterior las separa.

Es lo que hace *Certificate Transparency* con las autoridades de
certificación: no impide que se porten mal, **hace que no puedan hacerlo
en secreto**.

### Lo que detecta

| Ataque | |
|---|---|
| Borrar una operación del medio | ✅ |
| Alterar una entrada | ✅ |
| Alterar la entrada **y su resumen** | ✅ el encadenamiento lo propaga |
| Mostrar historiales distintos a dos partes | ✅ `first_divergence` los localiza |
| Borrar el registro reiniciando | ✅ persiste |
| Manipular el historial en una copia | ✅ `verify_chain` |

### Un hallazgo: abrir una cuenta es la única transición sin prueba

No se había visto en todo el proyecto, y salió porque el registro obliga a
que **cada** cambio de raíz deje entrada. `open_account` no genera prueba
—no crea dinero, la cuenta nace a cero— pero **sí mueve la raíz de
estado**.

Ahora deja entrada con resumen de prueba **cero**, y eso es visible para
quien verifique: sabe que esa transición está *registrada* pero no
*demostrada*.

### Un error de diseño corregido: la cadena no puede cruzar árboles

La primera versión encadenaba `custodian_root` para gobernanza y
`frozen_root` para congelación, mezclándolas con las de cuentas. **No
funciona**: la raíz de custodios de una entrada no tiene por qué ser la de
cuentas de la siguiente.

La cadena va **siempre sobre la raíz de cuentas**; las operaciones que no
la tocan tienen `root_old == root_new`, y lo que sí cambiaron queda atado
por el resumen de su prueba.

### ⚠️ Lo que sigue sin resolver

- **Nadie está obligado a mirar.** El registro permite detectar una
  bifurcación; que alguien la detecte depende de que haya observadores
  comparando copias.
- **El operador podría no publicar el registro.** Que exista no obliga a
  entregarlo.
- **No impide la censura.** Una operación que nunca se procesa no deja
  entrada, y su ausencia es indistinguible de que nunca se pidió.

## Congelación de cuentas: impuesta por el circuito, no por el operador

Un supervisor puede bloquear una cuenta bajo investigación.

```rust
let f = layer.set_frozen(&auth, cuenta, true)?;   // dos custodios
layer.apply_freeze(&f, cuenta)?;
```

### Por qué en circuito y no en la capa

Congelar desde la capa sería que el operador se niegue a procesar. Y **el
operador ya puede censurar cualquier operación**: eso no añadiría ninguna
garantía, solo le pondría nombre.

En circuito es distinto: **la prueba de liquidación acredita que el emisor
no está congelado** en esa raíz de estado. Cualquiera que la verifique lo
comprueba, sin confiar en el operador.

### El diseño: un árbol aparte, no un campo en la hoja

Añadir un indicador a la hoja de cuenta habría obligado a rehacer los seis
circuitos. En su lugar hay un **árbol de congelados** y la liquidación
demuestra **no-pertenencia** — la misma maquinaria que ya usa el doble
gasto.

**Profundidad 24**, porque su subida cabe en las **192 filas libres** del
circuito de liquidación. Con profundidad 32 habría hecho falta duplicar la
traza y con ella el coste de generar cada transferencia.

### Lo que impide y lo que no

| | |
|---|---|
| Gastar estando congelada | ❌ Impedido |
| **Recibir estando congelada** | ✅ **Permitido, a propósito** |
| Congelar sin dos custodios | ❌ Impedido |
| Descongelar más fácil que congelar | ❌ Simétrico |
| Levantar una congelación reiniciando | ❌ Persiste |

Que pueda **recibir** es deliberado: impedirlo dejaría fondos en el limbo
y rompería pagos legítimos hacia una cuenta bajo investigación.

### ⚠️ Lo que el circuito no puede saber

**Nada justifica la congelación.** Demuestra que dos custodios la
autorizaron, no que tuvieran razón. Y **no hay caducidad**: dura hasta que
alguien la levante.

### Dos fallos que encontraron los tests

**La persistencia guardaba el contador pero no el árbol.** Al reiniciar,
el número decía "hay una congelación" y ninguna cuenta lo estaba — peor
que no guardar nada, porque el estado quedaba contradictorio.

**Las instantáneas tampoco las incluían.** Restaurar una copia levantaba
todas las congelaciones. Se corrigió cambiando el formato a `ZKSSL2`, para
que una copia antigua **se rechace** en vez de cargarse a medias.

## Gobernanza: los custodios dejan de ser eternos

**El problema, que la recuperación agravó**: los custodios pueden emitir
dinero y —desde la recuperación— **reasignar cualquier cuenta**. Con un
conjunto inmutable, un custodio comprometido conserva ese poder para
siempre y la única salida es crear un ledger nuevo.

**La circularidad**: si los propios custodios autorizan cambiar el
conjunto, dos comprometidos pueden expulsar a los honestos y
perpetuarse.

### La solución: separar autoridades por nivel

| Conjunto | Puede | Mutabilidad |
|---|---|---|
| **Custodios** | Emitir, recuperar cuentas | **Cambiable** por gobernanza |
| **Gobernanza** | Cambiar los custodios | **Inmutable** |

```rust
let g = layer.update_custodians(&gov_auth, nuevo_conjunto)?;
layer.apply_governance(&g)?;
```

**La circularidad no desaparece: se traslada.** Si el conjunto de
gobernanza se compromete, no hay salida salvo crear un ledger nuevo.

Pero se traslada a claves que se usan **casi nunca** y pueden guardarse
sin conexión, frente a claves operativas expuestas a diario. Es una
mejora real, y no es lo mismo que resolver el problema.

### Opciones descartadas, y por qué

**Umbral más alto (3-de-N)**: reduce el riesgo, no lo elimina, y cada
firmante necesita su propio carril en la traza — el circuito se dispara.

**Retardo con veto**: en un nodo único **no protege nada**, porque el
operador controla el orden de las operaciones y por tanto el reloj.

### El cambio de semántica en la persistencia

Al abrir un ledger, el conjunto de **gobernanza se verifica** (es
inmutable) y el de **custodios se lee** (es mutable). Comparar el segundo
obligaría a que quien abre supiera de antemano cuál es el vigente — que
es justo lo que un cambio de gobernanza modifica.

Verificado en `the_current_custodian_set_survives_restart`: si al reabrir
se restaurara el original, **un custodio revocado recuperaría su poder
reiniciando el nodo**.

## Recuperación de cuenta: se acabó la pérdida irreversible

**El problema**: si una clave de gasto se comprometía, el dinero de esa
cuenta se perdía **para siempre**. Las demás carencias del sistema
degradan el servicio; esta destruía valor.

**Por qué la rotación voluntaria no basta**: cambiar la clave usando la
clave actual sirve para higiene, pero no para el compromiso — si el
atacante la tiene, también puede rotar, y ganaría la carrera.

**La solución**: recuperación asistida por los custodios que ya existen.

```rust
let nueva_id = derive_public_id(nueva_clave);   // el titular genera su clave
let r = layer.recover(&auth, cuenta, nueva_id)?; // dos custodios autorizan
layer.apply_recovery(&r, cuenta)?;
```

La API pide **la identidad nueva, no la clave**. El nuevo dueño nunca se
la entrega a quien opera el nodo — lo contrario sería contradictorio en
una operación que existe precisamente porque una clave se comprometió.

### ⚠️ El intercambio, dicho sin adornos

Se cambia *"pérdida irreversible si te roban la clave"* por **"dos
custodios pueden reasignar cualquier cuenta"**.

Es el intercambio correcto para un sistema bancario —un banco puede
reasignar bajo orden judicial— **pero solo si es visible**.

### El contador de recuperaciones

Por eso hay un **contador público**, atado en el circuito, que incrementa
en cada recuperación. Sin él, los custodios podrían reasignar cuentas en
silencio: desde fuera, una recuperación es indistinguible de cualquier
otra transición de estado.

**No impide el abuso —nada en un circuito puede— pero lo hace contable**,
que es la condición para que exista rendición de cuentas. Y persiste
entre reinicios, o un operador podría reiniciar el nodo para borrar el
rastro.

### Lo que garantiza el circuito

| | |
|---|---|
| Dos custodios **distintos** del conjunto | ✅ |
| **El saldo NO cambia** | ✅ |
| El contador incrementa exactamente en uno | ✅ |
| La clave comprometida deja de servir | ✅ |

Esa segunda es crítica: sin ella, dos custodios podrían **vaciar
cualquier cuenta bajo apariencia de recuperación**.

### Lo que el circuito NO puede verificar

**Que el nuevo titular sea el legítimo.** Eso lo comprueban los custodios
fuera de línea. La criptografía no sabe de quién es una cuenta.

Y **nada impide una carrera**: si el atacante gasta antes de que la
recuperación se aplique, ese dinero se va. La recuperación protege el
saldo restante, no lo ya gastado.

## Métricas de la capa, medidas

Todas en release, misma máquina, misma ejecución.

| Operación | Generar | Aplicar / verificar | Prueba |
|---|---|---|---|
| **Arranque** | **0,67 ms** | — | — |
| Emisión (2-de-N) | ~105 ms | ~2 ms | 57.342 B |
| Transferencia | ~620 ms | ~4 ms | 61.966 B |
| Destrucción | ~110 ms | ~2 ms | 54.924 B |
| Auditoría (banda) | ~250 ms | ~1,5 ms | 48.782 B |

**Verificar cuesta el 0,5-0,8% de generar.** Esa asimetría es la
propiedad económica que hace útil el sistema: quien recibe una
liquidación gasta dos órdenes de magnitud menos que quien la produce.

> ⚠️ **Esa razón es la de la AUDITORÍA, no la de la transferencia.**
>
> `verify_audit` **solo verifica**: 1,6 ms frente a 274 de generación, un
> **0,58 %**. Es la cifra correcta para el argumento que sostiene —un
> supervisor comprueba sin tocar el estado— pero **estaba atribuida a la
> transferencia**.
>
> Aplicar una transferencia cuesta **28,5 %** de generarla, porque `apply`
> **verifica, muta el árbol y escribe a disco**. No es comparable.
>
> Se detectó ejecutando `cargo test -p zk-ssl --release metrics --
> --nocapture` y comparando con lo publicado. Ver `AUDITORIA.md` §22.

**El arranque no genera claves.** En Groth16 eran 2,6 s de setup por
circuito. Aquí no hay nada que generar ni ningún secreto que destruir, y
eso se ve en el propio constructor.

### El límite cuantificado

**Mil transferencias: ~620 s de prueba y 120,4 MB acumulados.** Ese es el
argumento numérico a favor de las pruebas por lote — no una intuición.

⚠️ **Pero no es el límite más restrictivo.** La posición del nullifier se
deriva del propio nullifier, y por la paradoja del cumpleaños las
colisiones son probables a los **~65.000 pagos**. Ese es una parada
permanente para el afectado, no un coste acumulado. Ver `AUDITORIA.md`
§13.

### El coste por operación no crece

Cinco transferencias encadenadas: la última costó 1,18 veces la primera.
El árbol de nullifiers llenándose no degrada el rendimiento.

### ⚠️ El tamaño de prueba NO es constante, y eso mereció investigarse

Se suponía que era idéntico para cualquier importe. **No lo es**: varía
un **5,4%** (3.334 B sobre 61.517).

La causa probable es la deduplicación de nodos en los caminos de Merkle
de las posiciones consultadas, que salen de Fiat-Shamir y cambian de una
prueba a otra.

**Lo que importa no es la magnitud sino si correlaciona con el secreto.**
Si el tamaño creciera con el importe, un observador que solo viera las
pruebas podría inferir magnitudes.

| Escala | Correlación (16 muestras) |
|---|---|
| Importe absoluto | −0,196 |
| **log₂(importe)** | **+0,008** |

La logarítmica es la relevante: una fuga plausible dependería del número
de bits del importe. Ahí la correlación es esencialmente nula.

**Evidencia débil, no demostración.** Con 16 muestras se descarta una
fuga grosera; descartar la fuga exigiría centenares de muestras y un
análisis estadístico que no se ha hecho.

### ⚠️ Qué NO son estas cifras

Una sola ejecución en una máquina. Los tiempos de transferencia oscilaron
entre 180 y 620 ms según el contexto de caché. Sirven para comparar
órdenes de magnitud entre operaciones, **no como benchmark riguroso**.

## Ninguna vía para crear dinero

| Vía | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | Clave del emisor demostrada en circuito |
| Emisión encubierta | Suministro público atado en el circuito |
| Gastar dos veces | No-pertenencia demostrable |
| Gastar sin ser el titular | Autoridad de gasto |
| Reenviar una operación válida | Encadenamiento de raíces en la capa |

Cada una con su test discriminante — un testigo internamente coherente
que solo viola la restricción concreta. Tres veces durante el proyecto un
test negativo resultó no discriminar (fallaba por otra restricción) y hubo
que rehacerlo.

## Persistencia con verificación de integridad

El ledger se guarda en `sled` y se recupera al arrancar. Pero guardar y
cargar es lo fácil; lo que importa es **detectar un estado corrupto antes
de operar sobre él**.

Al abrir, la capa reconstruye los dos árboles desde las hojas
almacenadas, **recalcula sus raíces** y las compara con las del último
cierre. Si no coinciden, **el arranque falla**.

> Sin esa comprobación, el nodo generaría **pruebas perfectamente válidas
> de transiciones sobre un ledger que no es el real**. Desde fuera nadie
> lo notaría: las pruebas verificarían. Es el fallo más grave posible en
> un sistema de liquidación, y la única defensa es negarse a arrancar.

Verificado en `corrupted_ledger_is_detected_at_startup`.

**Los nullifiers gastados también persisten.** Sin eso, reiniciar el nodo
permitiría regastar todo lo anterior: el árbol volvería a estar vacío y
la no-pertenencia se satisfaría de nuevo. Toda la protección contra doble
gasto sería un espejismo entre reinicios.

**Los parámetros del sistema son inmutables.** Abrir un ledger existente
con otra identidad de emisor u otro límite regulatorio falla. Silenciarlo
permitiría **sustituir al banco central sin dejar rastro**.

## Lo que la revisión de auditoría destapó, y qué se hizo con ello

Ambas cosas aparecieron al escribir el modelo de amenaza de
[`AUDITORIA.md`](./AUDITORIA.md), no durante el desarrollo. Es un
argumento a favor de escribirlo aunque no haya auditor: **obliga a mirar
el sistema desde fuera**.

### `open_account` sin autorización → tope de cuentas ✅

Cualquiera podía crear cuentas hasta agotar la memoria del nodo. Ahora hay
un **tope inmutable**, como el de emisión.

⚠️ **Acota el daño, no impide el abuso.** Un atacante puede agotar el cupo
y dejar sin sitio a usuarios legítimos. Cerrarlo de verdad exigiría
autorización de custodio para abrir, y eso requiere un circuito nuevo:
**abrir hoy no genera ninguna prueba**.

### "El receptor no autoriza" → reetiquetado, no era el problema

Se anotó como fallo y al examinarlo **no lo era**: en efectivo y en banca
cualquiera puede darte dinero, y la mitigación universal es **devolver**,
no impedir. Construir prevención rompería el modelo — no podrías pagar a
alguien que no está conectado para autorizar.

**Lo que sí falta tiene otro nombre: congelar cuentas.** Un supervisor
necesita bloquear una cuenta bajo investigación. Eso exige un indicador
en la hoja, lo que cambia la estructura de los seis circuitos. No está
implementado.

## ⚠️ Lo que esta capa NO es

- **No hay red ni consenso.** Nodo único.
- **No hay copias ni replicación**, ni cifrado en reposo: quien tenga el
  disco ve todos los saldos.
- **Nada de esto ha sido auditado por terceros.**

---

# El trabajo comparativo que fundamentó la elección

# ZK-Sovereign Settlement Layer

Prueba de concepto real, verificada de extremo a extremo, de liquidación
de pagos bancarios con cumplimiento normativo demostrado mediante pruebas
de conocimiento cero — sin revelar saldos ni importes, vinculado a estado
real del ledger, con protección contra doble gasto.

**Todo lo que hay en este repositorio ha sido compilado y ejecutado de
verdad.** Cada afirmación de este README tiene una ejecución de test real
detrás, no una promesa. Donde algo no está resuelto, se dice
explícitamente — ver la sección de limitaciones más abajo.

## Qué demuestra este proyecto

Un circuito de cumplimiento que prueba, sin revelar los valores:
- `amount <= balance` (el emisor tiene fondos suficientes)
- `amount <= regulatory_limit` (la transacción no excede el límite normativo)
- El saldo corresponde a una cuenta real dentro de un árbol de Merkle del
  estado del ledger (no un valor inventado)
- La prueba no puede reutilizarse como si fuera una transacción nueva
  (nullifier con separación de dominio)

Todo ello integrado con un traductor de mensajería **ISO 20022** (pacs.008,
subconjunto simplificado — ver limitaciones).

## El ciclo monetario completo

| Operación | Autoridad | Suministro |
|---|---|---|
| `mint` | **Dos custodios distintos**, dentro del tope | Sube |
| `transfer` | **Titular** | **No cambia** |
| `burn` | **Titular** | Baja |

### Emisión con umbral: se acabó la clave única

Antes bastaba **una** clave para crear dinero. Robarla permitía emitir
hasta el tope. Ningún banco central opera así.

Ahora la emisión exige **dos custodios distintos** de un conjunto
comprometido en una raíz pública:

```rust
let (raiz, caminos) = build_custodian_set(&claves);
let mut layer = SovereignLayer::open("./ledger", raiz, limite, tope)?;
layer.mint(&auth, cuenta, importe)?;   // auth = dos custodios
```

**La capa ya no conoce ninguna clave de emisión**, solo la raíz del
conjunto.

#### El riesgo que hubo que cerrar: el mismo custodio contando dos veces

Lo difícil no es impedir que firme alguien de fuera —eso lo resuelve la
pertenencia al conjunto—. Lo difícil es que **un 2-de-N en el que un
custodio pueda duplicarse es un 1-de-N disfrazado**.

Se cierra en dos pasos que dependen el uno del otro:

1. **Índices estrictamente crecientes**, comprobado con un rango sobre
   `indice_b − indice_a − 1`.
2. **Los índices atados a los caminos** mediante un acumulador que los
   reconstruye desde los bits de dirección del Merkle. Sin esto el índice
   sería un número inventado y el paso 1 no valdría nada.

Verificado de forma aislada en `circuit_threshold` (11 tests) y en el
circuito de emisión completo (10 tests).

#### ⚠️ Qué garantía da exactamente, y cuál no

**Da**: robar una clave ya no basta. Se necesitan dos.

**No da**: que dos personas independientes hayan autorizado *esta*
emisión. En una arquitectura de nodo único, **quien genera la prueba
necesita las dos claves a la vez**, y si las tiene puede emitir lo que
quiera.

La autorización verdaderamente separada —cada custodio firmando desde su
propio HSM— requiere delegación de la prueba, que a su vez requiere la
descentralización que este proyecto no tiene.

**La garantía es "dos claves comprometidas en vez de una", no "dos
voluntades independientes".**

### El tope de emisión: ni siquiera el emisor puede inflar

`max_supply` es un **parámetro inmutable del ledger**, no de la
operación. Si el emisor pudiera declararlo en cada emisión, pondría el
que le conviniera y la restricción sería vacua.

El circuito demuestra `suministro_nuevo <= tope`, con el tope como
entrada pública. La autoridad emisora tiene la clave, pero **no puede
superar el tope sin crear un ledger nuevo** — y eso dejaría un rastro
imposible de ocultar.

Dos propiedades que van más allá del caso obvio:

- **El tope se aplica al acumulado**, no a cada emisión. Sin eso, mil
  emisiones pequeñas superarían cualquier límite
  (`the_cap_applies_to_the_accumulated_supply`).
- **Destruir libera capacidad de emisión**
  (`burning_frees_up_minting_capacity`). Eso define qué *es* el tope: un
  límite de **circulante**, no un contador histórico. Un contador
  histórico haría el sistema inutilizable a largo plazo — se agotaría
  aunque no quedara nada circulando.

Esa asimetría es lo que un banco central necesita ver: el dinero se mueve
sin crearse, y solo aparece o desaparece con una operación explícita que
lo registra en una cifra pública.

### Por qué destruir NO requiere la clave del emisor

**Destruir no puede crear dinero.** Reduce un saldo y el suministro en la
misma cantidad, así que la invariante global se preserva sin que el
emisor autorice nada.

Exigir su firma sería **política monetaria, no una garantía
criptográfica** — y tendría un efecto difícil de justificar: que el
titular no pudiera deshacerse de su propio saldo sin permiso. Si un
despliegue quisiera controlar la retirada de circulante, sería una capa
de política por encima, no una restricción del circuito.

**La identidad de quien destruye no es pública.** Desde fuera se sabe que
alguien con autoridad destruyó ese importe y que el suministro bajó en
consecuencia, pero no de quién era. La emisión sí identifica al emisor,
porque ahí la identidad de la autoridad es precisamente lo que hay que
comprobar.

Verificado en `full_monetary_cycle`: emitir → transferir → destruir,
comprobando la invariante en cada paso.

## ISO 20022: la capa habla el idioma de la banca

Sin esto el sistema es un motor sin conexión a nada. ISO 20022 es el
estándar sobre el que operan SWIFT, SEPA y TARGET2.

```text
pacs.008 (orden de transferencia)  →  pacs.002 (informe de estado)
                                        ACSC + prueba adjunta
                                        RJCT + código de motivo ISO
```

### Los errores se traducen a códigos ISO reales

| Situación | Código |
|---|---|
| Saldo insuficiente | `AM04` |
| Sobre el límite regulatorio | `AM02` |
| Divisa no admitida | `AM03` |
| IBAN desconocido | `AC01` |
| Clave de gasto incorrecta | `AG01` |
| Estado obsoleto | `DS0G` |

Un sistema receptor entiende `AM04` sin saber nada de esta
implementación. Eso es interoperar.

### Lo que la capa añade al estándar

El `pacs.002` lleva **la prueba y las raíces adjuntas**. ISO 20022 por sí
solo obliga al receptor a confiar en quien le envía el informe; aquí
puede verificar criptográficamente que la transición ocurrió.

### `settle_pacs008` nunca devuelve `Err`

Un rechazo es una respuesta válida del protocolo, no un error del
programa. Un sistema de mensajería espera **siempre** un informe de
estado; tratar el rechazo como excepción llevaría a mensajes perdidos y
operaciones en limbo.

### Lo que construir el puente destapó

`transfer` no comprobaba que quien firma fuese el titular, mientras `burn`
y `audit` sí. El circuito lo imponía igualmente —no había agujero— pero
la capa gastaba ~1 s generando una prueba que luego no verificaba, y
devolvía un error técnico en vez de decir que faltaba autorización.

**Solo apareció al construir la integración**, porque un puente obliga a
que cada error signifique algo preciso.

### ⚠️ Limitaciones

- **No es un parser XML**: es una struct con un subconjunto de campos.
- **La resolución IBAN → cuenta está fuera de la prueba.** El circuito
  demuestra que la transferencia entre dos posiciones es válida; que
  correspondan a esos IBAN lo garantiza el operador.
- **La clave de gasto no viaja en el mensaje** (ISO no transporta
  claves): viene de un almacén aparte.
- **Una sola divisa.** No hay `pacs.004` ni `camt.05x`.

## Privacidad y cumplimiento: revelación selectiva para el supervisor

El sistema demostraba cumplimiento del límite regulatorio dentro del
circuito, pero **un supervisor no podía auditar nada**: no había forma de
verificar el saldo de una cuenta concreta sin que la entidad enseñara
todo su estado.

Ese es el bloqueo real para adopción institucional. Un sistema
perfectamente privado en el que el regulador no puede comprobar nada no
es adoptable, por mucha matemática que tenga detrás.

`crates/zk-core/src/circuit_audit.rs` lo resuelve con dos formas de
revelación:

| | Qué demuestra | Restricciones |
|---|---|---|
| **Revelación exacta** | "mi saldo en el estado R es exactamente X" | 6.403 |
| **Revelación de rango** | "mi saldo supera X", **sin decir cuánto** | 7.225 |

La segunda es la interesante: un banco puede acreditar reservas mínimas o
requisitos de capital **sin exponer su posición**. Se demuestra el hecho
regulatorio relevante y ni un bit más.

**Auditar cuesta una séptima parte que transferir.** Un supervisor puede
verificar cientos de cuentas al coste de una sola operación — el perfil
que necesita un régimen de supervisión real.

### La decisión de diseño: revelación voluntaria, no custodia de claves

Había dos caminos:

**A)** Clave de visualización en poder del supervisor (el modelo de
"view keys" con escrow).

**B)** Revelación voluntaria: el titular produce la prueba dirigida a
quien se la pida.

**Se implementó B**, por riesgo sistémico: una clave custodiada es un
punto único de fallo. Quien la robe —un atacante, un empleado, un estado
hostil— ve la actividad de todo el sistema, retroactivamente y sin dejar
rastro. Con revelación voluntaria **no hay nada que robar**: cada
revelación es un acto deliberado, puntual y trazable.

**Contrapartida honesta**: el supervisor depende de la cooperación del
titular. Si una entidad se niega, la coerción viene de fuera
(requerimiento legal, sanción, suspensión de licencia), igual que hoy con
el secreto bancario. Esto no sustituye a la autoridad legal; le da una
herramienta criptográfica para verificar lo que se le entrega.

### Los tres tests que la hacen útil

- `lying_about_the_balance_is_rejected` — sin esto, la entidad declararía
  al supervisor lo que le conviniera.
- `third_party_cannot_disclose_someone_elses_balance` — sin la
  restricción de titularidad, cualquiera podría fabricar revelaciones
  falsas sobre terceros.
- `balance_below_threshold_is_rejected` — no se puede fingir solvencia.

9 tests. `cargo test -p zk-core --release circuit_audit`

## El límite regulatorio: corregido a medias, y dicho con claridad

**El agujero que había**: `regulatory_limit` era un argumento de
`transfer()` que viajaba dentro del `Settlement`, y `apply` verificaba la
prueba contra ese mismo valor declarado. **El probador elegía su propio
límite**; con `u64::MAX` la restricción se volvía vacua.

**La corrección aplicada**: el límite pasa a ser **parámetro del
sistema**, fijado al arrancar la capa. `apply` rechaza cualquier
liquidación que declare otro. Verificado en
`settlement_with_foreign_limit_is_rejected`, que manipula el límite a
`u64::MAX` y comprueba el rechazo.

**Lo que sigue sin resolverse**: ahora el límite lo fija *quien arranca
la capa*. Es mejor que dejarlo al regulado, pero **no hay una autoridad
reguladora criptográficamente distinguible del operador del nodo**.

La corrección completa es el "Governance Layer" de la arquitectura: un
regulador compromete los parámetros en una raíz pública, y cada
liquidación demuestra haber usado los comprometidos. Sería el quinto
circuito.

Se documenta la corrección parcial como tal porque es el tipo de matiz
que un supervisor detectaría en la primera revisión.

## Emisión: la última puerta por la que se podía crear dinero

El sistema dedicaba 15.522 restricciones a demostrar que el dinero se
conserva en cada transferencia. Pero `open_account` creaba cuentas **con
saldo, sin ninguna prueba**: el operador del nodo podía abrir una con mil
millones. Toda la conservación era irrelevante mientras existiera esa
puerta.

`crates/zk-core/src/circuit_mint.rs` la cierra separando **apertura** de
**emisión**:

```rust
layer.open_account(sk)                    // saldo CERO. No crea dinero, no necesita prueba.
layer.mint(issuer_key, cuenta, importe)   // EXIGE clave del emisor y prueba.
```

El circuito de emisión demuestra:

1. **Autoridad**: quien firma conoce la clave del emisor
   (`issuer_id = H(DOMAIN_ISSUER, issuer_key)`, con dominio separado del
   de las cuentas).
2. **La cuenta existe** con el saldo declarado.
3. **El saldo aumenta exactamente en `amount`.**
4. **El suministro público aumenta exactamente en `amount`.**

**15.016 restricciones** — un tercio que una transferencia. Emitir es
barato; transferir es caro, porque debe demostrar conservación entre dos
cuentas y no-pertenencia del nullifier.

### La conservación pasa de local a global

Con `total_supply` como valor público que solo crece mediante emisiones
demostradas, **cualquiera puede auditar que la suma de todos los saldos
equivale a lo emitido, sin ver un solo saldo**.

Verificado en `total_balances_always_equal_total_supply`, que además
comprueba que transferir no altera el suministro.

### Ya no queda ninguna vía para crear dinero

| Vía | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | Clave del emisor demostrada en circuito |
| Emisión encubierta | Suministro público, atado en el circuito |
| Gastar dos veces | No-pertenencia demostrable |
| Gastar sin ser el titular | Autoridad de gasto |
| Reenviar una operación válida | Encadenamiento de raíces en la capa |

17 tests en `settlement-layer`, 9 en `circuit_settlement`, 8 en
`circuit_mint`.

### Lo que este circuito NO resuelve

- **No hay destrucción de circulante (burn).** Un sistema real necesita
  retirar dinero; sería el circuito simétrico.
  Imponer reglas (techos, calendarios) sería otra capa.
- **La clave del emisor es única.** Un banco central real usaría umbral
  (m-de-n), no una sola clave.

## La capa ANTERIOR: `crates/settlement-layer`

> ⚠️ **Este crate está superado por `crates/zk-ssl`, y ningún otro depende
> de él.** `zk-ssl` lo cita como su predecesor:
>
> ```rust
> //! Equivalente del `settlement-layer::sparse_tree`, adaptado al backend
> ```
>
> Se conserva porque documenta cómo se llegó al diseño actual —incluidos
> **dos errores propios** que se cuentan más abajo— pero **no es la capa
> del sistema**. La capa es `zk-ssl`: 23 módulos y 172 tests, frente a los
> 2 módulos y 17 de este.
>
> Una versión anterior de este documento lo titulaba *"La capa"* sin más,
> lo que **contradecía a `AUDITORIA.md` §15** en el mismo repositorio.

Los demás crates implementan **circuitos**: demuestran que *una*
transferencia es válida. Este mantiene el **estado** y aplica operaciones
una tras otra. Fue lo que convirtió primitivas criptográficas en una capa
de liquidación **por primera vez** en este proyecto.

```rust
let mut layer = SettlementLayer::new(seed)?;
let alice = layer.open_account(sk_alice, 1_000_000);
let bob   = layer.open_account(sk_bob,      50_000);

let settlement = layer.transfer(sk_alice, alice, bob, 250_000, limite)?;
layer.apply(&settlement, alice, bob, 250_000)?;
```

**`transfer` genera la prueba sin tocar el estado; `apply` la verifica y
solo entonces aplica.** La separación es deliberada: permite que quien
produce la prueba y quien la acepta sean partes distintas, que es el caso
real entre bancos.

### Lo que la capa aporta y el circuito no puede

**Rechazo de repeticiones.** Reenviar una liquidación válida duplicaría
el dinero. El circuito no lo impide —la prueba sigue siendo válida—; lo
bloquea la capa al comprobar que `root_old` es el estado actual.
Verificado en `replaying_a_settlement_is_rejected`.

**Encadenamiento de estado.** Cada operación parte de la raíz que dejó la
anterior, con nonce y nullifier distintos. Verificado en
`consecutive_transfers_chain_correctly`.

**Un árbol disperso viable.** El de `zk-core::merkle` reconstruye 2^20
hojas en cada cambio: sirve para tests, es inviable para un ledger.
`SparseMerkleTree<DEPTH>` guarda solo las ocupadas y actualiza en `DEPTH`
hashes. La profundidad va en el tipo (20 para cuentas, 32 para
nullifiers) porque mezclarlas ya provocó un fallo, y así lo impide el
compilador.

**17 tests.** Ejecutar: `cargo test -p settlement-layer --release`

⚠️ Una versión anterior de este documento decía **12**, y otro punto decía
**16**. Ninguna de las dos era cierta.

### Dos errores propios que los tests destaparon

**`apply` no insertaba el nullifier.** El árbol nunca crecía, así que la
no-pertenencia —la garantía más cara del sistema, 15.522 restricciones—
habría sido **vacua en la práctica**. Lo detectó
`full_transfer_cycle_updates_state` al comprobar que la raíz cambia.

**Hacer el nullifier privado fue un error de diseño.** Se presentó como
mejora de privacidad; no lo era. Sin conocerlo, quien aplica la
liquidación no puede mantener su árbol. En Zcash los nullifiers son
públicos exactamente por eso: la privacidad viene de que son
**indistinguibles** (derivados de `sk`, sin vínculo con ninguna cuenta),
no de ocultarlos. Revertido.

### ⚠️ Lo que esta capa NO es

- **No hay red ni consenso.** Es un nodo único en memoria. Una federación
  real necesita acuerdo sobre el orden de las operaciones — un problema
  de sistemas distribuidos, no de criptografía.
- **No hay persistencia del árbol de cuentas.**
- **`open_account` no está demostrada.** Modifica el árbol directamente:
  es administración, no operación de usuario. Sin un circuito de emisión,
  **el operador puede crear dinero**.
- **No hay delegación de la prueba.** Quien la genera necesita la clave
  de gasto.
- **Solo sobre Groth16.**

## Autoridad de gasto: de prueba de solvencia a capa de liquidación

`crates/zk-core/src/circuit_settlement.rs` es **el circuito que debe
usarse**. Los anteriores (`circuit_with_state`, `circuit_double_entry`)
se conservan como referencia de la evolución del diseño, pero tienen un
agujero grave:

> **Demostraban que quien ejecuta el circuito CONOCE los datos de la
> cuenta, no que esté AUTORIZADO a gastar.** Cualquiera con acceso al
> saldo, el nonce y el camino de Merkle —de una filtración, un backup, un
> empleado— podía mover el dinero.

El diseño corregido sigue el modelo de Zcash Sapling:

```text
sk        = clave de gasto (privada, nunca sale del titular)
pk        = H(DOMAIN_PK,   sk)
leaf      = H(H(pk, balance), nonce)
nullifier = H(H(DOMAIN_NULL, sk), nonce)
```

**Dos propiedades nuevas:**

**Autorización.** El circuito exige conocer `sk` tal que `pk = H(sk)`. La
identidad de la cuenta pasa a ser un compromiso criptográfico a la clave.

**Inobservabilidad del nullifier.** Antes se derivaba de un identificador
público, así que cualquiera podía **precomputar el nullifier de una
cuenta ajena y vigilar el registro de gastados para saber cuándo esa
cuenta mueve dinero**. En una red interbancaria eso revela el patrón de
operaciones de todos los participantes a cualquier observador.
Derivándolo de `sk`, solo el titular puede calcularlo.

**Coste medido: 475 restricciones, un 1,7% del circuito.**

| | Restricciones | Setup | Prueba |
|---|---|---|---|
| Partida doble sin autoridad | 27.562 | 1,12 s | 1,17 s |
| **Circuito de liquidación** | **28.037** | **2,59 s** | **1,68 s** |

Verificado con 8 tests, entre ellos
`attacker_without_spend_key_cannot_transfer`: un atacante que conoce todo
de la cuenta salvo la clave no puede transferir.

### La asimetría emisor/receptor es deliberada

Solo el emisor demuestra autoridad; el receptor aparece con su identidad
pública, sin clave. **Recibir dinero no requiere permiso** — exigir la
firma del receptor haría imposible transferir a quien no esté presente
para autorizar. Misma asimetría que en Zcash y en cualquier sistema de
pagos real.

## No-pertenencia demostrable: el doble gasto deja de depender de una base de datos

`persistent_nullifier_registry` es una base de datos `sled` de un solo
nodo, y el circuito **no comprobaba nada sobre ella**. Quien controlara
ese nodo podía aceptar un nullifier repetido: **la prueba seguiría siendo
criptográficamente válida**. Era el único punto del sistema donde una
parte podía crear dinero sin romper ninguna matemática.

`crates/zk-core/src/nullifier_tree.rs` lo cierra con un árbol disperso de
32 niveles. Al gastar, el circuito demuestra **dos cosas a la vez**:

1. **No-pertenencia**: la posición del nullifier estaba vacía en
   `nullifier_root_old`.
2. **Inserción**: `nullifier_root_new` es ese árbol con el nullifier
   insertado.

Ambas raíces son públicas, así que la cadena es auditable. **El doble
gasto pasa de "detectable por una base de datos" a "matemáticamente
imposible".**

**Efecto colateral favorable**: el nullifier deja de ser entrada pública
— queda comprometido dentro del árbol. Menos información expuesta con la
misma garantía.

### El coste de cada garantía, medido

| | Restricciones | Incremento |
|---|---|---|
| Partida doble sola | 27.562 | — |
| + autoridad de gasto | 28.037 | +475 (**1,7%**) |
| **+ no-pertenencia demostrable** | **43.559** | **+15.522 (55%)** |

La garantía más fuerte del sistema es también, con diferencia, la más
cara: cerrar el doble gasto criptográficamente cuesta **33 veces más**
que cerrar la autorización.

Verificado con 9 tests, entre ellos
`double_spend_is_rejected_by_the_circuit`.

### ⚠️ Limitación documentada: colisiones de posición

La posición se deriva de los 32 bits bajos del nullifier (un árbol sobre
todo el campo tendría 254 niveles). Dos nullifiers pueden colisionar: el
segundo **no podría gastarse**. Es **denegación de servicio, no doble
gasto** — la solidez se mantiene, la completitud no.

Con 10.000 nullifiers la probabilidad ronda 1 entre 10⁵. Aceptable para
una demostración; a escala haría falta un árbol indexado (como el de
Aztec). Hay un test ejecutable
(`position_collisions_are_possible_and_documented`) que lo demuestra en
vez de dejarlo en un comentario.

### Lo que sigue faltando para producción

- **No hay delegación de la prueba.** Generar una prueba cuesta ~1,7 s y
  requiere `sk`. En un banco, la clave viviría en un HSM y el cómputo lo
  haría otro servicio — pero ese servicio necesitaría la clave, y con
  ella podría enviar el dinero a donde quisiera. La solución (firmar los
  detalles con `sk` y verificar la firma DENTRO del circuito) exige
  aritmética de curva elíptica en circuito, una pieza mucho mayor.
- **No hay revelación selectiva** para auditoría del supervisor.
- **Solo está implementado en Groth16.** Los otros cuatro backends siguen
  con el diseño sin autoridad ni no-pertenencia, así que la comparativa
  presenta como equivalentes cosas que ya no lo son.

## El núcleo contable: partida doble

El circuito principal (`ComplianceCircuitWithState`) demuestra que el
emisor tiene saldo suficiente y respeta el límite regulatorio. Pero **no
demuestra qué pasa con el dinero**: nada ata el adeudo del origen al
abono del destino.

`crates/zk-core/src/circuit_double_entry.rs` cierra esa carencia. Prueba
la transición de estado completa:

```text
saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
```

La conservación no es una tautología: **ambos saldos están comprometidos
en el árbol de Merkle y ambas raíces (antes y después) son públicas**, así
que el mismo importe que se resta de un saldo comprometido se suma a otro
saldo comprometido, y el resultado se refleja en una raíz pública.

Verificado con tests que construyen deliberadamente escenarios de
**creación** y **destrucción** de dinero (el receptor recibe 10.000 más o
menos de lo debitado) y comprueban que el circuito los rechaza — casos que
el circuito anterior no podía detectar.

| | Circuito con estado | Circuito de partida doble |
|---|---|---|
| Restricciones | 9.934 | 27.562 |
| Setup (release) | 438 ms | 1.123 ms |
| Prueba (release) | 422 ms | 1.170 ms |
| Demuestra | solvencia del emisor | **conservación del dinero** |

Reproducir: `cargo test -p zk-core --release circuit_double_entry -- --nocapture`

### Implementado en los CUATRO backends

| | Groth16 | Halo2/IPA | STARK | PLONK-KZG |
|---|---|---|---|---|
| Módulo | `zk-core::circuit_double_entry` | `halo2-experiment::circuit_double_entry` | `stark-experiment::double_entry` | `plonk-experiment::circuit_double_entry` |
| Tamaño | 27.562 restricciones | k = 17 | 113 restricciones, 41 columnas, 1024 filas | 84.801 puertas |
| Tests | 8/8 | 6/6 | 8/8 | 7/7 |

**El port a STARK reveló un problema estructural que no existe en los
otros dos paradigmas**: AIR carece de restricciones de copia, así que
nada obliga a que las dos subidas del árbol (hoja antigua y hoja nueva)
usen los mismos hermanos. Un diseño secuencial habría permitido usar
hermanos distintos en cada subida y fabricar una raíz que no corresponde
a la misma posición del árbol — un agujero **silencioso**, porque los
testigos honestos sí usan los mismos hermanos.

La solución es un diseño en **lockstep**: los dos carriles avanzan nivel
a nivel a la vez y una restricción fuerza que el hermano sea idéntico en
ambos. Verificado de forma aislada en `stark-experiment::dual_climb`,
con un test que construye dos carriles internamente coherentes por
caminos distintos — exactamente el ataque que el diseño secuencial
permitiría.

Conclusión práctica: **portar un circuito de Plonkish a AIR no es
mecánico, ni siquiera cuando la lógica es idéntica.**

**Limitaciones**: no gestiona creación de moneda por un banco central
(que rompería la conservación deliberadamente y necesitaría una entrada
pública de emisión); y no impide una autotransferencia con una
restricción dedicada, aunque la rechaza por construcción.

## Cinco motores criptográficos, verificados por separado

Este proyecto implementa la misma lógica **cuatro veces**, con paradigmas
y esquemas de compromiso distintos (R1CS, Plonkish con IPA, AIR con FRI,
y Plonkish con KZG), más un **quinto backend de paradigma diferente**
(Nova / plegado), para comparar sus trade-offs con datos medidos en vez
de con lo que dicen los papers:

| | `zk-core` + `iso-bridge` | `halo2-experiment` | `stark-experiment` |
|---|---|---|---|
| Esquema | Groth16 (Arkworks, BLS12-381) | Halo2 con IPA (Zcash, Pallas) | STARK/FRI (Winterfell, Goldilocks) |
| Paradigma | R1CS | Plonkish | AIR |
| Trusted setup | Sí, por circuito — ceremonia MPC implementada (ver abajo) | No | No |
| Setup / parámetros | 438 ms | 16,3 s | ninguno |
| Generación | 422 ms | 4,86 s | 38 ms |
| Verificación | 5 ms | 91 ms | 1 ms |
| Tamaño de prueba | 192 bytes | 4.096 bytes | 36,7 KB / 125,6 KB |
| Resistencia cuántica | No | No | Sí (solo hashes) |
| Árbol de Merkle | 20 niveles | 20 niveles | 32 niveles |
| `SettlementProver` | Implementado | Implementado | Implementado |
| Estado | Verificado end-to-end | Verificado end-to-end | Verificado end-to-end |

Las dos cifras de tamaño del backend STARK corresponden a dos niveles de
seguridad distintos, y la diferencia importa: **36,7 KB con 127 bits
_conjeturados_** (el nivel que se cita habitualmente en el ecosistema
STARK) frente a **125,6 KB con 128 bits _demostrables_** (solidez con
demostración, sin apoyarse en la conjetura de decodificación de
Reed-Solomon). El puente ISO 20022 del backend STARK usa la segunda por
defecto.

**La comparación completa de los tres, con metodología y números
medidos, está en [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md).** La
comparación previa de dos vías, con el análisis de gobernanza
institucional, sigue disponible en
[`GROTH16_VS_HALO2.md`](./GROTH16_VS_HALO2.md). Las métricas detalladas
del motor Groth16 están en [`PERFORMANCE.md`](./PERFORMANCE.md).

### El quinto backend: Nova, un paradigma distinto

`crates/nova-experiment` no compite en el mismo eje que los otros cuatro.
Ellos producen una **prueba monolítica** del circuito entero; Nova
**pliega** una secuencia de pasos y comprime al final.

La pregunta que responde y los demás no: **¿cuánto cuesta la transacción
número N+1?** Medido: **~250 ms, constante** — el paso 9 costó 0,77 veces
el paso 1. El cierre (comprimir a una prueba entregable) cuesta 1,84 s,
amortizables entre todas las transacciones de la jornada.

**Alcance honesto**: se cerró en nivel de prueba de concepto. No
implementa el circuito de cumplimiento ni la partida doble. Y su
sobrecoste fijo es sustancial — 10.764 restricciones por paso para un
circuito que hace un solo hash.

**Un hallazgo colateral que merece la pena**: `nova-snark` es la única de
las cinco librerías que **impide en código** generar parámetros con una
sola parte, y ofrece `setup_with_ptau_dir` para consumir ficheros de
Perpetual Powers of Tau. Las otras cuatro lo permiten y confían en la
documentación.

### Nota sobre el hallazgo más relevante del backend STARK

Sin extensión de campo, la solidez tiene un techo de **63 bits** — el
tamaño del propio campo Goldilocks — por muchas queries que se añadan. La
configuración "rápida y compacta" que uno elegiría por defecto **no es
comparable** con los ~128 bits de Groth16 o Halo2. Es el tipo de detalle
que desaparece de los materiales promocionales y que aquí está medido y
documentado.

## Estructura del repositorio

```
crates/
  zk-core/                circuito Groth16 (ComplianceCircuit, ComplianceCircuitWithState)
  iso-bridge/              traductor ISO 20022 -> testigos del circuito Groth16
  halo2-experiment/        mismo circuito, reimplementado en Halo2/IPA
    src/range_check.rs           range check de 64 bits
    src/poseidon_hash.rs         Poseidon real via halo2_gadgets
    src/merkle.rs                 arbol de Merkle de 20 niveles
    src/nullifier.rs              nullifier con separacion de dominio
    src/compliance_circuit.rs     los cuatro anteriores, unificados
    src/compliance_real_proof.rs  pipeline de prueba real con IPA + metricas
    src/iso_bridge.rs             traductor ISO 20022 -> Halo2
    src/persistent_nullifier_registry.rs  persistencia en sled
  stark-experiment/        mismo circuito, reimplementado en STARK/AIR
    src/range_check.rs            range check de 63 bits (techo de Goldilocks)
    src/rescue_hash.rs            permutacion Rescue Prime como restricciones AIR
    src/merkle.rs                 arbol de Merkle de 32 niveles
    src/nullifier.rs              nullifier con separacion de dominio
    src/solvency.rs               solvencia con valores privados (Horner)
    src/compliance_circuit.rs     todo unificado en una sola traza de 512 filas
    src/compliance_real_proof.rs  pipeline real + metricas de 4 configuraciones
    src/iso_bridge.rs             traductor ISO 20022 -> STARK (sin claves)
    src/persistent_nullifier_registry.rs  persistencia en sled
```

## Cómo compilar y verificar

```bash
# Motor Groth16 (zk-core + iso-bridge)
cargo test --workspace --no-fail-fast -- --nocapture

# Motor Halo2 (aislado, dependencias distintas)
cargo test -p halo2-experiment -- --nocapture

# Motor STARK (aislado, ecosistema winterfell)
cargo test -p stark-experiment -- --nocapture

# Metricas STARK reales (requiere release para que los tiempos sean citables)
cargo test -p stark-experiment real_proof --release -- --nocapture

# Un test del backend STARK esta marcado #[ignore] por una traza
# degenerada (valor cero) que dispara un falso positivo de una assertion
# de depuracion de winterfell. Ejecutarlo asi:
cargo test -p stark-experiment --release -- --ignored
```

**Aviso de tiempo real**: los tests que generan pruebas criptográficas
completas (no solo comprobación de satisfacibilidad) tardan de varios
segundos a varios minutos, según la pieza y la contención de CPU del
entorno. Esto es esperado, documentado, y no indica ningún problema. Ver
`PERFORMANCE.md` y `GROTH16_VS_HALO2.md` para tiempos concretos medidos.

## Limitaciones honestas — léase antes de considerar esto para producción

- **Ceremonia MPC: mecanismo resuelto, ceremonia no celebrada.** El
  motor Groth16 necesita un trusted setup por circuito. El crate
  `crates/ceremony` implementa la ceremonia MPC de dos fases (BGM17) y
  está verificado sobre el circuito real: `cargo test -p zk-core
  --release --test ceremony_integration`. **Pero ejecutar las
  contribuciones en una sola máquina demuestra que el mecanismo
  funciona, NO que exista seguridad**: la garantía MPC ("basta con que
  un participante sea honesto y destruya su aleatoriedad") requiere
  participantes reales e independientes publicando el transcript. Ese
  paso está pendiente. Ver `crates/ceremony/ATTRIBUTION.md`.

  Nota histórica: este proyecto documentó durante mucho tiempo que no
  existía tooling de ceremonia para Arkworks 0.4.x, tras investigar
  `ark-marlin` y `celo-org/snark-setup`. Esa conclusión era **errónea o
  quedó obsoleta**: Penumbra publicó `penumbra-sdk-proof-setup`, del que
  procede el código de `crates/ceremony`.
- **Registro de nullifiers persistente pero de un solo nodo**, en ambos
  motores. Resuelve la persistencia ante reinicios de un validador, no la
  coordinación entre varios validadores de un consorcio — eso requiere
  que el registro forme parte del estado replicado por consenso.
- **`iso-bridge` es un subconjunto simplificado** de pacs.008.001.08, no
  un parser XML conforme al XSD real, y no maneja decimales específicos
  por divisa (JPY, KWD, etc. difieren de EUR/USD).
- **Los tres backends NO son intercambiables entre sí.** Cada uno opera
  sobre un cuerpo finito distinto, así que sus árboles de Merkle y sus
  espacios de nullifiers son incompatibles: una cuenta registrada en el
  árbol de Groth16 no existe en el de STARK. El trait
  `SettlementProver` unifica la FORMA de la llamada, no los datos — ver
  la nota extensa en `crates/settlement-prover/src/lib.rs`.
- **El backend STARK cubre 63 bits de rango, no 64**, por el tamaño del
  campo Goldilocks (2^64 - 2^32 + 1, menor que 2^64). Suficiente para
  cualquier importe monetario real, pero es una diferencia con los otros
  dos motores, no una equivalencia.
- **La resistencia post-cuántica del backend STARK no es gratuita.**
  Descansa en que la función hash resista el algoritmo de Grover, lo que
  en la práctica exige doblar el tamaño de salida respecto al caso
  clásico. Es un ajuste mucho más simple que parchear curvas elípticas
  con retículas, pero no es "sin parches".
- **Nada de este código ha sido auditado externamente.**
- **`ark-groth16` se declara oficialmente "prototipo académico, no listo
  para producción"** por sus propios mantenedores.

## Licencia

Este proyecto se publica bajo doble licencia MIT / Apache-2.0, a elección
de quien lo use — la misma convención que usan Arkworks y Halo2, los dos
ecosistemas sobre los que está construido. Ver `LICENSE-MIT` y
`LICENSE-APACHE`.

## Contribuir / reportar problemas

Este es un proyecto de investigación e ingeniería abierto. Si encuentras
un error, una vulnerabilidad, o quieres proponer una mejora (en
particular: la ceremonia MPC pendiente, la coordinación distribuida del
registro de nullifiers, o el parser ISO 20022 completo), las
contribuciones son bienvenidas.
