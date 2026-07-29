# ZK-SSL — ¿Puede interesar a bancos centrales e instituciones financieras?

**Evaluación honesta, no material comercial.**

Este documento responde a las preguntas que haría un equipo técnico de un
banco central, un consorcio Tier-1 o una infraestructura de mercado. Cada
respuesta separa **lo que existe y está medido** de **lo que haría falta**.

Está escrito así por una razón práctica: en una conversación con esos
interlocutores, cualquier afirmación no verificable se detecta en la
primera revisión del repositorio, y contamina todo lo demás. Un documento
que declara sus límites es el único que sobrevive a ese escrutinio.

---

## Respuesta corta

**¿Puede interesar como infraestructura desplegable?** No. No hay red, no
hay consenso, no ha sido auditado y no ha procesado nunca una operación
real.

**¿Puede interesar como evidencia técnica?** Sí, en un punto concreto:
**aporta mediciones reproducibles sobre lo que cuesta la privacidad
criptográfica con cumplimiento demostrable**, y ocho hallazgos de diseño
que no están documentados en otro sitio.

Esa distinción es la diferencia entre una conversación que avanza y una
que termina.

---

## 1. ¿Es una infraestructura modular, descentralizada y post-cuántica?

Tres adjetivos, tres respuestas distintas.

### Modular — **sí**

Ocho circuitos independientes, capa de estado separada de los circuitos,
generación de pruebas separada de la aplicación de las mismas, y puente
ISO 20022 desacoplado del núcleo. La arquitectura permite sustituir el
sistema de prueba sin rehacer la lógica de negocio: de hecho, el mismo
circuito está implementado en cinco.

### Descentralizada — **NO**

**Es un nodo único.** Quien lo opera:

- Ve todos los saldos.
- Ordena las operaciones y puede censurar.

Ambas cosas exigen consenso distribuido, que **no está implementado** y es
un problema de sistemas distribuidos, no de criptografía. Se estima en
20-40 semanas de trabajo especializado.

Lo único cerrado en esa dirección: un **registro encadenado de
transiciones** que impide reescribir el historial en secreto. Publicar 32
bytes compromete toda la historia. Es la construcción de *Certificate
Transparency* — no impide el mal comportamiento, lo hace detectable.

### Post-cuántica — **sí, y es la propiedad más sólida**

El núcleo criptográfico depende **exclusivamente de funciones hash**
(Rescue-Prime sobre el campo Goldilocks, con FRI). No hay curvas elípticas
ni emparejamientos en el camino crítico.

Es relevante para un horizonte de despliegue de infraestructura
financiera, donde el sistema debe seguir siendo seguro décadas después de
su puesta en marcha.

**Matiz técnico que conviene conocer**: la seguridad declarada de 127 bits
es **conjeturada**. La seguridad *demostrable* en esa configuración está
entre 29 y 63 bits. Cerrar la brecha eleva el tamaño de prueba de 36,7 KB
a 125,6 KB. Esa distinción rara vez se explicita, y es directamente
relevante para elegir parámetros bajo criterios regulatorios.

### Identidad soberana — **parcialmente**

Hay claves de gasto e identidades derivadas criptográficamente con
separación de dominio: una clave de custodio no puede hacerse pasar por
titular de cuenta, ni al revés.

**No hay DIDs, ni credenciales verificables, ni SSI en el sentido del
estándar W3C.** El componente de identidad del planteamiento original está
implementado a nivel de autoridad criptográfica, no de identidad digital.

---

## 2. ¿Resuelve la paradoja entre liquidación instantánea y secreto bancario?

**Demuestra que la paradoja es resoluble, y mide lo que cuesta. No la
resuelve en producción porque no hay producción.**

### Lo que está demostrado

Una operación puede ser **privada frente a terceros** —que solo ven una
prueba— y simultáneamente **verificable por un supervisor**, sin que este
tenga acceso al ledger ni a ninguna clave maestra.

El mecanismo es revelación selectiva con tres modos:

| Modo | El titular demuestra | El supervisor aprende |
|---|---|---|
| Exacto | Su saldo es X | El saldo |
| Mínimo | Su saldo supera X | Solo eso |
| **Banda** | Su saldo está entre X e Y | Solo el rango |

El modo de banda satisface un requisito habitual de supervisión
—confirmar que una posición está dentro de un rango— **sin revelar la
cifra**.

### La propiedad estructural que lo hace distinto

La prueba la genera **el titular**. El supervisor la verifica con una
función libre.

**No existe ninguna clave de custodia que robar** para obtener acceso
general a los saldos. No hay puerta trasera de supervisión.

La contrapartida está declarada: si el titular se niega a cooperar, el
sistema no ofrece mecanismo de revelación forzosa. La supervisión es
cooperativa, no coercitiva. Para un despliegue regulado, eso es una
decisión de política que habría que evaluar.

### Lo que NO está resuelto

**No hay liquidación transfronteriza** porque no hay capa de red. El
sistema opera sobre un solo nodo; no existe protocolo entre instituciones,
ni atomicidad entre jurisdicciones, ni gestión de corresponsalía.

**Sí hay interoperabilidad de mensajería**: la capa consume `pacs.008` y
devuelve `pacs.002` con la prueba adjunta y códigos de motivo ISO 20022
reales (`AM04` saldo insuficiente, `AC01` cuenta incorrecta, `AG01`
transacción prohibida...). Un sistema receptor entiende el rechazo sin
conocer esta implementación.

Eso es una pieza de interoperabilidad real, pero **no es liquidación
transfronteriza**.

---

## 3. ¿Qué tecnología criptográfica utiliza como núcleo?

**STARK/FRI sobre el campo Goldilocks**, con la función hash Rescue-Prime,
aritmetización AIR.

### Por qué esa elección, y no la más rápida

Se evaluaron cinco paradigmas implementando el mismo circuito en cada uno:

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG |
|---|---|---|---|---|
| Ceremonia de confianza | Por circuito | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño de prueba | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Se descartó Groth16 pese a ser más rápido y producir pruebas 320 veces
menores.**

El motivo es el que más debería importar a un banco central: Groth16 y
PLONK/KZG **exigen una ceremonia de setup de confianza**. Un conjunto de
participantes genera parámetros a partir de un secreto que debe
destruirse. Si coluden y lo conservan, **pueden falsificar pruebas y crear
dinero sin dejar rastro detectable**. Las pruebas falsas verifican
correctamente; no hay mecanismo posterior de detección.

Para una institución cuyo mandato incluye la soberanía monetaria, esa
dependencia es permanente —no caduca— e inauditable —no se puede comprobar
que el secreto se destruyó—.

Es la única decisión del proyecto tomada **contra** los números de
rendimiento.

---

## 4. ¿Dónde opera físicamente la infraestructura?

**En una máquina de desarrollo. No hay despliegue.**

No existe infraestructura operativa: ni centros de datos, ni nodos
distribuidos, ni entorno de producción, ni acuerdos de nivel de servicio.

El sistema es un conjunto de bibliotecas en Rust más una capa de estado
con persistencia local. Se ejecuta donde se compile.

### Lo que existe en materia operativa

| Capacidad | Estado |
|---|---|
| Persistencia con verificación de integridad al arrancar | ✅ |
| Escrituras atómicas (un lote por operación) | ✅ |
| Instantáneas exportables con verificación | ✅ |
| Cifrado en reposo (XChaCha20-Poly1305) | ✅ |
| Replicación en vivo | ❌ |
| Alta disponibilidad | ❌ |
| Recuperación ante desastres automatizada | ❌ |

El cifrado en reposo tiene una consecuencia operativa que conviene
conocer: **la clave la aporta el operador al arrancar**, así que el nodo
no puede reiniciar sin intervención. Guardarla junto a los datos no
protegería nada.

---

## 5. ¿Dónde se procesan las pruebas de conocimiento cero?

**En el cliente**, y esa es una decisión de diseño relevante.

```
1. El cliente pide la vista pública de su cuenta
2. Calcula el nullifier LOCALMENTE, con su clave
3. Pide los materiales: caminos de Merkle y datos de estado
4. Genera la prueba EN SU MÁQUINA
5. Envía la liquidación; la capa verifica y aplica
```

**La clave de gasto nunca llega al operador del nodo.** Un atacante que
intercepte los materiales —caminos de Merkle, saldos, nonces— no puede
generar la prueba sin la clave.

### Coste computacional

| Operación | Generar | Verificar |
|---|---|---|
| Transferencia | ~620 ms | ~4 ms |
| Emisión | ~105 ms | ~2 ms |
| Auditoría | ~250 ms | ~1,5 ms |

**Verificar cuesta el 0,5-0,8% de generar.** Esa asimetría es la propiedad
económica que hace viable el modelo: el coste recae en quien produce la
prueba, no en quien la acepta.

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

### La limitación

Generar una prueba requiere ~620 ms y memoria significativa. Un cliente
ligero —un terminal de punto de venta, un móvil— no puede hacerlo
cómodamente.

Delegarlo a un tercero **sin entregarle la clave** exige verificar una
firma dentro del circuito. Es técnicamente viable con firmas basadas en
hash (Winternitz, coherentes con el diseño post-cuántico) pero **no está
implementado**: se estima en unas 8.000 filas adicionales de traza.

---

## 6. ¿Dónde se almacenan los estados globales y las cuentas?

**En una base de datos embebida local** (`sled`), en la misma máquina que
ejecuta el nodo.

### Estructura del estado

| Estructura | Profundidad | Contenido |
|---|---|---|
| Árbol de cuentas | 32 (4.290 M posiciones) | `H(H(identidad, saldo), nonce)` |
| Árbol de nullifiers | 32 | Marcas de gasto |
| Árbol de congelados | 24 (16,7 M) | Cuentas bloqueadas |

Los árboles son **dispersos**: solo se materializan las hojas ocupadas, y
los subárboles vacíos se representan por su raíz canónica. La memoria es
proporcional al uso real, no al espacio de direcciones.

Además hay escalares públicos: suministro total, límite regulatorio, tope
de emisión, raíces de los conjuntos de custodios y gobernanza, y tres
contadores de intervención.

### Garantías del almacenamiento

**Verificación de integridad al arrancar.** Se reconstruyen los árboles
desde las hojas guardadas, se recalculan las raíces y se comparan con las
del último cierre. Si no coinciden, **el arranque falla**.

Sin esa comprobación, el nodo generaría pruebas perfectamente válidas de
transiciones sobre un ledger que no es el real — criptográficamente
indetectable desde fuera.

**Escrituras atómicas.** Cada operación escribe cuentas, nullifiers,
metadatos y registro en un solo lote. Si el proceso muere en medio, el
ledger queda coherente: se pierde la operación, no la integridad.

**Cifrado en reposo.** Los valores se cifran con XChaCha20-Poly1305
autenticado. Protege contra el robo del disco o de una copia; **no contra
el operador**, que ve los saldos en memoria.

### Las limitaciones de escala, cuantificadas

Son **cuatro**, y no se compensan entre sí. La más restrictiva no es la de
almacenamiento.

#### 1. Colisión de posiciones de nullifier — **la primera en morder**

La posición de un nullifier **se deriva del propio nullifier**, y el
circuito exige que esté libre. Dos pagos distintos que caigan en la misma
posición son un conflicto, y eso sigue la paradoja del cumpleaños:

| Pagos acumulados | Probabilidad de colisión |
|---|---|
| 10.000 | 1,2 % |
| **65.536** | **39 %** |
| 200.000 | **99 %** |

⚠️ **El afectado no puede reintentar.** Su nullifier es determinista a
partir del estado de su cuenta: **su pago queda bloqueado de forma
permanente**.

**No es un coste, es una parada.** Y a diferencia de las otras tres, le
ocurre a un usuario concreto sin que el sistema esté saturado.

#### 2. Agotamiento del árbol de pendientes

El contador de posiciones **nunca reutiliza** las liberadas al reclamar, así
que el límite es de transferencias **totales desde el inicio**: 2³². A mil
pagos por segundo, **unos cincuenta días**.

Ahora falla declarando su causa —`PendingTreeExhausted`— en vez de producir
una prueba que no verifica.

#### 3. Acumulación de pruebas

**Mil transferencias acumulan 120,4 MB.** Es un coste de almacenamiento y
ancho de banda, no una parada: el sistema sigue funcionando.

#### 4. Tamaño del conjunto de custodios

El orden estricto entre custodios se comprueba con un segmento de 7 bits,
lo que **limita el conjunto a 128 miembros**. Con más, las autorizaciones
entre índices lejanos fallarían de forma intermitente.

---

⚠️ **Una versión anterior de este documento decía que los 120,4 MB eran "el
límite real del sistema".** Era falso: el primero de esta lista detiene
pagos legítimos mucho antes, y de forma permanente.

Resolver el tercero exige agregación recursiva o pruebas por lote. Los
otros tres exigen decisiones de diseño distintas. **Ninguno está resuelto**;
los cuatro están documentados en `AUDITORIA.md` §13.

---

## 7. ¿Cómo se garantiza la privacidad de las transacciones?

### Lo que un tercero NO puede ver

Ni identidades, ni saldos, ni importes. Solo ve una prueba criptográfica y
las raíces de estado públicas, que son compromisos y no revelan su
contenido.

**El nullifier solo lo puede calcular el titular**, porque se deriva de su
clave de gasto. Eso impide a un observador precomputar los nullifiers de
cuentas ajenas para vigilar cuándo gastan.

### Un hallazgo sobre el tamaño de prueba

Se comprobó si el tamaño de la prueba filtra información sobre el importe.
**No es constante**: varía un 5,4% entre operaciones.

La correlación medida entre importe y tamaño es **+0,008 en escala
logarítmica** —esencialmente nula—, lo que descarta una fuga grosera.

⚠️ Con 16 muestras eso es **evidencia débil, no demostración**. Descartar
la fuga exigiría centenares de muestras y análisis estadístico, que no se
ha hecho. Se documenta como pregunta abierta.

### ⚠️ Lo que el operador SÍ ve

**Todos los saldos.** La capa mantiene el estado, luego lo conoce.

La privacidad de este sistema es **frente a terceros que solo ven
pruebas**, no frente a quien mantiene el ledger.

Para una institución, eso significa que el modelo actual es aplicable a un
escenario donde **la entidad que opera el nodo tiene legítimamente acceso
a los datos** —un banco central sobre sus propios registros, por ejemplo—
pero **no** a un escenario multiinstitucional donde los participantes no
deban verse entre sí.

Ese segundo escenario exige consenso, y con él, replicación del estado
entre partes que no confían entre sí.

---

## 8. ¿Cómo se asegura la resistencia a ataques cuánticos?

**Por construcción**: el núcleo depende exclusivamente de funciones hash.

### Qué usa y qué no

| Componente | Primitiva | ¿Vulnerable a cuántica? |
|---|---|---|
| Compromiso de estado | Rescue-Prime (hash) | No |
| Sistema de prueba | FRI (hash + Reed-Solomon) | No |
| Identidades y nullifiers | Rescue-Prime | No |
| Cifrado en reposo | XChaCha20-Poly1305 | Reducción a la mitad de la seguridad efectiva (Grover), asumible |
| **Curvas elípticas** | **Ninguna en el camino crítico** | — |

Los algoritmos de Shor rompen los problemas del logaritmo discreto y la
factorización, en los que se basan las curvas elípticas y los
emparejamientos. **Groth16, PLONK/KZG, Halo2 y Nova los usan; STARK/FRI
no.**

Grover reduce a la mitad la seguridad efectiva de las funciones hash, lo
que está contemplado en la elección de parámetros.

### La cautela que corresponde

La resistencia cuántica del sistema de prueba es una propiedad de la
construcción, no una promesa. Pero conviene recordar el matiz del
apartado 1: **los 127 bits declarados son conjeturados**; los demostrables
en esa configuración están entre 29 y 63.

Para un despliegue bajo criterios regulatorios, esa distinción debería
resolverse explícitamente al elegir parámetros, y el coste está medido:
36,7 KB frente a 125,6 KB por prueba.

---

## 9. Entonces, ¿en qué podría interesar realmente?

Con todo lo anterior sobre la mesa, hay tres respuestas honestas.

### Como fuente de datos para una decisión de paradigma

Cualquier institución que evalúe conocimiento cero para liquidación tiene
que elegir sistema de prueba. Este trabajo aporta **mediciones
reproducibles del mismo circuito en cinco paradigmas**, con hallazgos que
no aparecen en la literatura:

- La aritmetización AIR **carece de restricciones de copia**, lo que abre
  un agujero de solidez silencioso al implementar actualizaciones de
  estado. No se descubre midiendo SHA-256.
- El campo Goldilocks es **demasiado estrecho para identidades**: 64 bits
  son colisión en 2³².
- La **brecha entre seguridad conjeturada y demostrable**.
- **PLONK/KZG resultó el generador más lento** de los cuatro basados en
  curvas.

### Como respuesta cuantificada a "¿cuánto cuesta la privacidad?"

El debate sobre privacidad en monedas digitales de banco central está
lleno de posiciones y escaso de cifras. Aquí las hay: **verificar cuesta 4
ms, generar 620, y mil transferencias son 59 MB**.

Y hay un dato del propio BCE que enmarca el problema: en su consulta
pública, la privacidad fue el aspecto más valorado (43%), y **menos de una
de cada diez respuestas apoyaban el anonimato pleno**. Privacidad sin
anonimato total es exactamente lo que hace la revelación selectiva.

### Como demostración de que el problema es tratable

Que las propiedades sean construibles simultáneamente —privacidad,
cumplimiento demostrable, conservación del dinero, auditoría selectiva,
sin ceremonia de confianza— no era evidente antes de construirlo. Ahora
está medido y es reproducible.

---

## 10. Qué haría falta para un piloto institucional

Por orden de dependencia:

| Requisito | Estado | Estimación |
|---|---|---|
| **Auditoría externa** | No realizada | Condición previa a todo lo demás |
| **Consenso distribuido** | No implementado | 20-40 semanas |
| Delegación de prueba a terceros | No implementado | 4-8 semanas |
| Agregación de pruebas | No implementado | 6-12 semanas |
| Replicación y alta disponibilidad | No implementado | 3-6 semanas |
| Rotación operativa de claves | Parcial | 2-4 semanas |
| Caducidad y justificación de congelaciones | No implementado | 2-4 semanas |

**La auditoría externa es la única que es una condición, no una
capacidad.** Ninguna cantidad de pruebas propias la sustituye, y sin ella
lo demás no debería desplegarse.

---

## 11. Verificar todo lo anterior

Nada de este documento requiere confianza en su autor.

```bash
git clone [repositorio]
cd zk-ssl
cargo test -p zk-ssl --release              # 170 tests
cargo test -p stark-experiment --release    # 199 tests
cargo test -p zk-ssl --release metrics -- --nocapture
```

Requiere únicamente Rust estable. Sin instaladores externos ni cadenas de
herramientas aparte.

El repositorio incluye un documento de preparación para auditoría
(`AUDITORIA.md`) con el modelo de amenaza, la tabla de invariantes y dónde
se impone cada una, y **una sección explícita con los puntos donde el
autor tiene menos confianza**.

---

## Nota final

Este documento no vende nada. Está escrito para que un equipo técnico
pueda decidir en veinte minutos si merece la pena una conversación, y para
que esa decisión sea correcta en ambos sentidos.

Si la respuesta es que no —porque falta consenso, porque falta auditoría,
porque no es infraestructura— **esa es la respuesta correcta hoy**, y es
preferible obtenerla ahora que después de una reunión construida sobre
afirmaciones que no se sostienen.
