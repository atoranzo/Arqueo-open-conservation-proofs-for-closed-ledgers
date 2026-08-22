# Implementación comparativa de una capa de liquidación con conocimiento cero en cinco sistemas de prueba: hallazgos de diseño y mediciones

**Nota sobre este borrador.** Está escrito en español. arXiv lo admite,
pero la audiencia de `cs.CR` lee mayoritariamente en inglés y una versión
española reduce la difusión de forma notable. Recomiendo traducirlo antes
de enviarlo. Además, arXiv exige **aval** (*endorsement*) para publicar
por primera vez en la mayoría de categorías: hay que conseguirlo aparte.

---

## Resumen

Presentamos la implementación del mismo circuito de liquidación
financiera en cinco sistemas de prueba de conocimiento cero —Groth16,
Halo2/IPA, STARK/FRI, PLONK/KZG y Nova/plegado— y la evaluación
comparativa resultante. A diferencia de los trabajos comparativos
existentes, que miden circuitos de referencia como SHA-256, la comparación
se realiza sobre una **aplicación completa**: una capa de liquidación con
partida doble, autoridad de gasto, prevención de doble gasto, emisión con
umbral, destrucción de circulante, revelación selectiva para supervisión
y congelación de cuentas.

Esa diferencia metodológica resulta determinante. Documentamos ocho
hallazgos que no aparecen en la literatura comparativa y que solo emergen
al portar una aplicación con estado entre paradigmas. El más significativo
es que la aritmetización AIR **carece de restricciones de copia**, lo que
abre un agujero de solidez silencioso al implementar actualizaciones de
árboles de Merkle, invisible para testigos honestos y ausente en las
aritmetizaciones Plonkish y R1CS.

Reportamos mediciones de generación, verificación y tamaño de prueba
obtenidas en condiciones idénticas, y documentamos un error metodológico
propio —mezcla de compilaciones de depuración y optimizadas— detectado y
corregido durante el trabajo.

La implementación de referencia consta de **957 pruebas ejecutables en la
compuerta de sello** —1094 contando los pines de los niveles largo y
completo, y 1108 declaradas—, con 13 ignoradas y declaradas, y
está disponible públicamente. **No ha sido auditada por terceros y no
implementa consenso distribuido**; discutimos en detalle las implicaciones
de ambas limitaciones.

**Palabras clave**: pruebas de conocimiento cero, STARK, liquidación
financiera, aritmetización AIR, ceremonias de setup, resistencia cuántica.

---

## 1. Introducción

### 1.1 Motivación

Los sistemas de liquidación financiera presentan una tensión estructural
entre dos requisitos aparentemente incompatibles. Por un lado,
confidencialidad: los participantes no deben poder observar las posiciones
ni las operaciones de terceros. Por otro, cumplimiento normativo
verificable: un supervisor debe poder comprobar que se respetan límites,
que no se crea dinero fuera de los cauces autorizados y que las
operaciones son legítimas.

Las soluciones desplegadas resuelven esa tensión mediante confianza
institucional: una entidad central observa todo y certifica que se cumplen
las reglas. Las pruebas de conocimiento cero permiten en principio una
solución distinta —demostrar el cumplimiento sin revelar los datos— pero
su aplicación a liquidación financiera plantea decisiones de diseño cuyas
consecuencias no están bien documentadas.

Este trabajo aborda una pregunta concreta: **¿qué cambia realmente al
implementar la misma aplicación de liquidación sobre paradigmas de prueba
distintos?**

### 1.2 Por qué los benchmarks existentes no responden a esa pregunta

Existe trabajo comparativo riguroso sobre sistemas de prueba. zk-Bench
evalúa Groth16, PLONK, halo2 y starky con metodología cuidadosa, y
reporta, entre otros resultados, que las implementaciones de circuitos a
medida pueden distorsionar el rendimiento percibido de una librería.

Sin embargo, esos trabajos miden **circuitos de referencia**: funciones
hash, exponenciación modular, verificación de firmas. Son comparaciones
válidas de rendimiento bruto, pero no capturan las decisiones de diseño
que impone cada paradigma cuando la aplicación tiene **estado
persistente**, **invariantes globales** y **múltiples autoridades**.

Nuestra hipótesis de partida —confirmada por los resultados— es que las
diferencias más consecuentes entre paradigmas no son de rendimiento sino
de **expresividad y de riesgos de solidez**, y que solo se manifiestan al
implementar una aplicación completa.

### 1.3 Contribuciones

1. **Implementación del mismo circuito de liquidación en cinco
   paradigmas**, con mediciones en condiciones idénticas (§7).
2. **Ocho hallazgos de diseño** ausentes en la literatura comparativa
   (§8), entre ellos la falta de restricciones de copia en AIR y su
   consecuencia para actualizaciones de estado.
3. **Una capa de liquidación completa** con propiedades monetarias
   verificables, jerarquía de autoridades y supervisión por revelación
   selectiva (§4-§6).
4. **Documentación explícita de las limitaciones**, incluidos errores
   propios detectados y corregidos (§9), y del método de verificación
   empleado (§10).

### 1.4 Lo que este trabajo NO aporta

Por claridad, y porque la delimitación es parte de la contribución:

- **No propone primitivas criptográficas nuevas.** Emplea construcciones
  establecidas.
- **No implementa consenso distribuido.** La arquitectura es de nodo
  único, con las consecuencias que se analizan en §11.
- **No constituye una auditoría de seguridad.** El sistema no ha sido
  revisado por terceros.
- **Las mediciones son de una sola ejecución en una máquina.** Sirven para
  comparar órdenes de magnitud, no como benchmark riguroso.

---

## 2. Tecnología: sistemas de prueba y su elección

### 2.1 El criterio que resultó determinante

Los cinco paradigmas evaluados difieren en varias dimensiones —tamaño de
prueba, tiempo de generación, tiempo de verificación, hipótesis
criptográficas— pero una resultó decisiva para esta aplicación: **si
requieren o no una ceremonia de setup de confianza**.

Groth16 exige una ceremonia por circuito; PLONK/KZG, una ceremonia
universal reutilizable. En ambos casos, un conjunto de participantes
genera parámetros a partir de un valor secreto que debe destruirse. **Si
todos los participantes coluden y conservan ese secreto, pueden falsificar
pruebas.**

En una aplicación de liquidación, esa capacidad se traduce en algo
concreto: **crear dinero sin dejar rastro detectable**. Las pruebas
falsificadas verifican correctamente. No existe mecanismo posterior de
detección.

Para una infraestructura cuyo propósito declarado incluye la
independencia de terceros, esa dependencia es estructuralmente
incompatible: es permanente —no caduca— e inauditable —no se puede
comprobar que el secreto se destruyó—.

### 2.2 La decisión, tomada contra los números

De los cinco paradigmas, dos prescinden de ceremonia: Halo2/IPA y
STARK/FRI. De esos dos, STARK/FRI resulta superior en generación y
verificación, e incorpora una propiedad adicional: **resistencia
cuántica**, al depender exclusivamente de funciones hash.

El coste es sustancial. Las pruebas STARK del circuito de comparación
ocupan 36,7 KB frente a los 192 bytes de Groth16: un factor de
**320×**. Los circuitos de producción de la capa son mayores —**53,6 a
65,3 KB**, medidos en §218—, así que el factor real ronda 300-350×. En tiempo de generación
del circuito completo, la diferencia favorece a STARK, pero el tamaño
condiciona cualquier escenario donde las pruebas deban transmitirse o
almacenarse en volumen.

Es, en nuestro trabajo, la única decisión de diseño tomada explícitamente
**contra** los resultados de rendimiento.

### 2.3 Aritmetización: R1CS, Plonkish y AIR

Los tres modelos de aritmetización empleados difieren en cómo expresan las
restricciones, y esa diferencia tiene consecuencias que analizamos en §8.

**R1CS** (Groth16) expresa el cómputo como restricciones de rango uno.
Cada variable existe una vez; la igualdad entre variables es directa.

**Plonkish** (Halo2, PLONK) organiza el cómputo en una tabla con
restricciones de puerta y **restricciones de copia** (*copy constraints*)
que fuerzan la igualdad entre celdas arbitrarias de la tabla.

**AIR** (STARK) expresa el cómputo como restricciones sobre transiciones
entre filas consecutivas de una traza de ejecución. **No dispone de
restricciones de copia**: la única relación expresable directamente es
entre una fila y la siguiente.

Esa ausencia, que en circuitos sin estado pasa desapercibida, resultó ser
el hallazgo más significativo de este trabajo (§8.1).

---

## 3. Arquitectura

### 3.1 Visión general

El sistema se estructura en tres niveles:

**Circuitos** (aritmetización AIR sobre el campo Goldilocks). Ocho
circuitos que demuestran las propiedades de cada tipo de operación:
liquidación, emisión, destrucción, auditoría, umbral, recuperación,
gobernanza y congelación.

**Capa de estado.** Mantiene los árboles de Merkle, encadena las raíces
entre operaciones, verifica las pruebas y aplica las transiciones. No
conoce ninguna clave privada.

**Cliente.** Genera las pruebas. **La clave de gasto no sale de la máquina
del titular**: la capa entrega los caminos de autenticación y el cliente
construye la prueba localmente.

> ⚠️ **Nota de corrección (cuarta revisión).** Esta propiedad era, hasta el
> 31 de julio de 2026, una propiedad **del diseño** y no del sistema: la
> capa **no verificaba las pruebas** de la vía de pago antes de aplicar la
> transición, de modo que la clave no hacía falta para mover fondos ajenos.
> Las revisiones anteriores la enunciaron sin esa salvedad. El defecto está
> corregido y medido; el registro completo, en `AUDITORIA.md` §73.

### 3.2 Separación entre generar y aplicar

Todas las operaciones se dividen en dos actos:

```
generar → produce una prueba, NO modifica el estado
aplicar → verifica la prueba y, si es válida y corresponde
          al estado actual, la aplica
```

La separación es deliberada y tiene una consecuencia relevante: permite
que **quien produce la prueba y quien la acepta sean partes distintas**.
Es la condición para que un supervisor externo verifique una liquidación
sin acceso al ledger, y para que la generación de pruebas ocurra en el
cliente.

### 3.3 El estado

El estado consta de tres árboles de Merkle dispersos y varios escalares
públicos:

| Estructura | Profundidad | Contenido |
|---|---|---|
| Árbol de cuentas | 32 | Hoja = `H(H(id, saldo), nonce)` |
| Árbol de pendientes | 32 | Compromisos enviados y sin cobrar |
| Árbol de congelados | 24 | Cuentas bloqueadas |

Escalares públicos: suministro total, límite regulatorio, tope de emisión,
raíz del conjunto de custodios, raíz del conjunto de gobernanza, y tres
contadores de intervención (recuperaciones, cambios de gobernanza,
congelaciones).

La profundidad 24 del árbol de congelados no es arbitraria: es la máxima
cuya subida cabe en las filas libres del circuito de liquidación sin
duplicar la longitud de la traza, lo que habría duplicado el coste de
generación de toda transferencia.

---

## 4. Estructuras de Merkle y sus restricciones

### 4.1 Árboles dispersos

Los tres árboles son dispersos: solo se materializan las hojas ocupadas, y
los subárboles vacíos se representan por su raíz canónica precomputada.
Con profundidad 32, el espacio de direcciones es de 2³² posiciones, pero
la memoria es proporcional al número de hojas realmente ocupadas.

Esa representación permite generar caminos de autenticación tanto para
posiciones ocupadas como libres, y esa segunda capacidad es la base de las
pruebas de **no-pertenencia**.

> ⚠️ **Un límite de capacidad que existió, y cómo desapareció.**
>
> En la vía de un paso, la posición de un nullificador se derivaba del
> propio nullificador —`nullifier[0] mod 2³²`— y el circuito exigía que
> estuviera libre. Dos nullificadores distintos en la misma posición eran
> un conflicto, y eso seguía la **paradoja del cumpleaños**: a los
> ~65.000 la colisión tenía un 39 % de probabilidad, y un 99 % con
> 200.000. El afectado no podía reintentar —el nullificador es
> determinista a partir del estado de su cuenta—, así que **su pago
> quedaba bloqueado**: un límite de disponibilidad, no de solidez.
>
> Los circuitos en dos fases **no usan nullificador** (§9), y con la
> retirada de la vía de un paso el árbol quedó sin nada que lo escribiera
> y se eliminó de la capa. **El límite ya no aplica.**
>
> ⚠️ **Pero se evitó, no se resolvió.** Lo que sustituye al nullificador
> es el encadenamiento de raíces, y eso **exige un orden total**, que un
> nodo único da y un sistema distribuido no. Quien distribuya esto
> recupera el problema entero. Ver `AUDITORIA.md` §13, §32 y §36.
>
> Los árboles de cuentas, pendientes y congelados nunca tuvieron este
> problema: sus posiciones **se asignan**, no se derivan.

### 4.2 No-pertenencia como primitiva

Dos propiedades del sistema se demuestran mediante no-pertenencia, con la
misma técnica:

| Propiedad | Árbol | Se demuestra que… |
|---|---|---|
| La cuenta no está congelada | Congelados | su hoja es cero |
| La posición del pendiente estaba libre | Pendientes | su hoja era cero antes de insertar |

⚠️ **El doble gasto ya no se cierra así.** En la vía de un paso era una
tercera no-pertenencia —la posición del nullificador libre—; hoy lo cierra
el **encadenamiento de raíces**, con la dependencia del orden total que eso
implica (§4.1 y `AUDITORIA.md` §36).

La técnica consiste en subir desde una hoja **cero** hasta la raíz
declarada. Si la posición estuviera ocupada, su hoja no sería cero y la
subida no alcanzaría esa raíz.

### 4.3 Actualización de estado: el patrón en lockstep

La operación central del sistema —una transferencia— exige demostrar una
transición de estado: que a partir de una raíz conocida, modificando dos
hojas, se llega a otra raíz.

La implementación natural consiste en dos subidas del árbol: una con la
hoja antigua, otra con la nueva. En aritmetizaciones con restricciones de
copia, basta con forzar que ambas subidas usen los mismos hermanos.

En AIR eso no es expresable directamente, y su ausencia abre un agujero
que analizamos en §8.1. La solución adoptada —dos carriles paralelos en la
traza, con una restricción que impone la igualdad de hermanos en cada
fila de enlace— es lo que denominamos **patrón en lockstep**.

---

## 5. Propiedades monetarias verificables

### 5.1 El ciclo completo

| Operación | Autoridad requerida | Efecto en el suministro |
|---|---|---|
| Emisión | Dos custodios distintos, dentro del tope | Aumenta |
| Transferencia | Titular de la cuenta | **No varía** |
| Destrucción | Titular de la cuenta | Disminuye |

La asimetría entre transferencia y las otras dos es la propiedad
verificable central: **el dinero se mueve sin crearse**, y solo aparece o
desaparece mediante operaciones que lo registran en una cifra pública.

### 5.2 Vías de creación de dinero y su cierre

Enumeramos exhaustivamente las vías por las que un adversario podría crear
dinero, y la restricción que cierra cada una:

| Vía | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | Dos custodios demostrados en circuito |
| Emisión sin reflejo en el suministro | Suministro público atado en el circuito |
| Superar el tope de emisión | Rango sobre `tope − suministro` |
| Gastar dos veces | Encadenamiento de raíces (orden total del nodo único) |
| Gastar sin ser titular | Autoridad de gasto demostrada |
| Reenviar una operación válida | Encadenamiento de raíces |

### 5.3 Autoridad de umbral: la garantía y su límite

La emisión exige dos custodios distintos de un conjunto comprometido en
una raíz pública. El riesgo no trivial no es que firme alguien externo
—eso lo cierra la prueba de pertenencia— sino que **el mismo custodio
cuente como dos**, lo que convertiría un esquema 2-de-N en un 1-de-N
encubierto.

Se cierra mediante dos restricciones interdependientes:

1. **Índices estrictamente crecientes**: se demuestra por rango que
   `índice_b − índice_a − 1 ≥ 0`.
2. **Los índices están atados a los caminos**: un acumulador reconstruye
   cada índice a partir de los bits de dirección del camino de Merkle
   demostrado.

La segunda es indispensable: sin ella, el índice sería un valor declarado
sin relación con la posición realmente demostrada, y la primera no
garantizaría nada.

**Límite de la garantía.** En una arquitectura de nodo único, quien genera
la prueba necesita ambas claves simultáneamente. La garantía obtenida es
por tanto **"dos claves comprometidas en lugar de una", no "dos voluntades
independientes"**. La autorización genuinamente separada —cada custodio
firmando desde su propio módulo de seguridad— requiere verificar firmas
dentro del circuito, lo que no está implementado.

---

## 6. Supervisión: revelación selectiva

### 6.1 El mecanismo

Un único circuito demuestra que `inferior ≤ saldo ≤ superior`. Variando
los parámetros se obtienen tres modos de revelación:

| Modo | Configuración | Se revela |
|---|---|---|
| Exacta | `inferior = superior = saldo` | El saldo |
| Mínimo | `inferior = X`, `superior = MAX` | Que supera X |
| Banda | `inferior = X`, `superior = Y` | Que está entre X e Y |

El modo de banda permite satisfacer un requisito de supervisión
—confirmar que una posición está en un rango— sin revelar la cifra.

### 6.2 Propiedad estructural: no hay claves de custodia

La prueba la genera **el titular** con su clave. El supervisor la verifica
mediante una función libre, sin acceso al ledger ni a ninguna clave
maestra.

La consecuencia es que **no existe ninguna clave que robar** para obtener
acceso general a los saldos: no hay puerta trasera de supervisión. La
contrapartida es que el supervisor depende de la cooperación del titular;
si este se niega, el sistema no ofrece mecanismo de revelación forzosa.

Esa contrapartida es deliberada y conviene declararla: convierte la
supervisión en un proceso cooperativo, no coercitivo.

### 6.3 Congelación de cuentas

Un supervisor puede bloquear una cuenta mediante autorización de dos
custodios. La propiedad relevante es **dónde se impone la restricción**.

Si la impusiera únicamente la capa de estado, sería equivalente a que el
operador se negara a procesar la operación —una capacidad que ya posee—
y no añadiría ninguna garantía verificable por terceros.

En nuestra implementación, **la prueba de liquidación acredita que el
emisor no pertenece al árbol de congelados** en esa raíz de estado.
Cualquier verificador lo comprueba sin confiar en el operador.

Una cuenta congelada conserva la capacidad de **recibir**. Impedirlo
dejaría fondos en un limbo y rompería pagos legítimos hacia una cuenta
bajo investigación.

### 6.4 Contadores de intervención

Las tres operaciones que otorgan poder discrecional a los custodios
—recuperación de cuenta, cambio del conjunto de custodios y congelación—
incrementan un **contador público atado en el circuito**.

Los contadores no impiden el abuso: ninguna restricción de circuito puede.
Lo hacen **contable**, que es la condición necesaria para que exista
rendición de cuentas.

---

## 7. Evaluación comparativa

### 7.1 Metodología

- Mismo circuito de cumplimiento implementado en los cinco paradigmas.
- Todas las mediciones en compilaciones optimizadas.
- Misma máquina, misma sesión de ejecución.
- Ninguna cifra procede de la literatura.

**Limitación explícita**: una sola ejecución por medición, sin control de
varianza ni caracterización del hardware. Los tiempos observados de una
misma operación variaron entre 180 y 620 ms según el estado de caché. Las
cifras son válidas para comparar órdenes de magnitud entre paradigmas, no
como benchmark.

### 7.2 Resultados

| | Groth16 | Halo2/IPA | STARK/FRI | PLONK/KZG |
|---|---|---|---|---|
| Aritmetización | R1CS | Plonkish | AIR | Plonkish |
| Ceremonia | Por circuito | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño de prueba | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Nova/plegado** se evalúa por separado por su naturaleza distinta: no
produce una prueba final entregable, sino un estado plegado que requiere
compresión posterior. Coste marginal por transacción: **~250 ms,
constante** (el paso 9 costó 0,77× el paso 1). Cierre: 1,84 s
amortizable.

### 7.3 Mediciones de la capa completa

| Operación | Generar | Verificar | Prueba |
|---|---|---|---|
| Arranque | **0,67 ms** | — | — |
| Emisión (2-de-N) | ~105 ms | ~2 ms | 57.342 B |
| Transferencia | ~620 ms | ~4 ms | 61.966 B |
| Destrucción | ~110 ms | ~2 ms | 54.924 B |
| Auditoría (banda) | ~250 ms | ~1,5 ms | 48.782 B |

**Asimetría verificar/generar: 0,5-0,8%.** Es la propiedad económica que
hace viable el modelo: el coste recae en quien produce la prueba, no en
quien la acepta.

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

**Límite de escala cuantificado**: mil transferencias acumulan 126,2 MiB de
pruebas. Es la restricción práctica dominante de la elección de STARK, y
el argumento cuantitativo a favor de agregación recursiva o pruebas por
lote.

---

## 8. Hallazgos

### 8.1 AIR carece de restricciones de copia

**El hallazgo principal.** Al portar la actualización de estado a AIR
aparece un agujero de solidez inexistente en R1CS y Plonkish.

La actualización requiere dos subidas del árbol de Merkle —hoja antigua y
nueva— desde la misma posición. Nada en la aritmetización AIR obliga a que
ambas subidas empleen los mismos nodos hermanos. Un probador malicioso
podría emplear caminos distintos y producir una raíz nueva que no
corresponde a modificar esa posición del árbol original.

**El agujero es silencioso**: un testigo honesto usa siempre los mismos
hermanos, de modo que ninguna prueba legítima lo revela. Solo aparece al
analizar qué restricciones existen y cuáles no.

La solución adoptada —el patrón en lockstep de §4.3— tiene un coste
concreto: duplica el ancho de la traza en la fase de subida.

**Implicación general**: portar un circuito de Plonkish a AIR no es una
operación mecánica, ni siquiera cuando la lógica es idéntica. Las
propiedades que en Plonkish se obtienen mediante restricciones de copia
deben rediseñarse.

### 8.2 El campo Goldilocks es demasiado estrecho para identidades

Un elemento del campo Goldilocks son 64 bits. Una identidad de cuenta
representada por un solo elemento admite colisión con **2³² operaciones**
por el argumento del cumpleaños: computacionalmente trivial.

En BLS12-381 (255 bits por elemento) el problema no se plantea, lo que
explica que no aparezca al diseñar sobre esa curva. La corrección consiste
en emplear digests completos de cuatro elementos (256 bits).

⚠️ **Esa corrección es necesaria y no suficiente, y las revisiones
anteriores la presentaron como completa.** Ensanchar la **identidad** impide
hallar *otra* clave que colisione con ella; **no impide hallar *la* clave**.
Si el secreto sigue siendo un solo elemento, su espacio es 2⁶⁴ y la
identidad es pública, de modo que agotarlo por fuerza bruta fuera de línea
cuesta 2⁶³ —medido en 2,38 millones de años-núcleo sobre una CPU sin
optimizar el ataque, que es **cota superior floja**—.

El criterio que §8.3 aplica al techo de solidez —insuficiente frente a los
~128 bits de los otros paradigmas— **se aplica igual al espacio de claves**,
y las revisiones anteriores no lo hicieron. La corrección completa exige
**cuatro elementos también en el secreto**; está implementada y medida
(`AUDITORIA.md` §82, §90, §97), y **la migración es opt-in**: las claves
generadas antes de rotar siguen teniendo 64 bits.

### 8.3 Techo de solidez en STARK sobre Goldilocks

Sin extensión de campo, la configuración que un implementador elegiría por
defecto —rápida y compacta— tiene un techo de **63 bits de solidez**,
insuficiente y no comparable con los ~128 bits de los otros paradigmas.

### 8.4 Brecha entre seguridad conjeturada y demostrable

En las configuraciones evaluadas, 127 bits de seguridad **conjeturada**
conviven con 29-63 bits **demostrables**. Cerrar la brecha eleva el tamaño
de prueba de 36,7 KB a 125,6 KB.

La distinción rara vez se explicita en comparaciones entre paradigmas, y
es directamente relevante para quien deba elegir parámetros bajo criterios
regulatorios.

### 8.5 PLONK/KZG resultó el generador más lento

Contraintuitivo para un sistema frecuentemente presentado como estándar de
la industria: 16-22× más lento que Groth16 en generación.

**Matiz metodológico**: parte de la diferencia puede atribuirse a la
implementación (`dusk-plonk` frente a `arkworks`) y nuestros datos no
permiten separar ambos efectos. Es exactamente el fenómeno que zk-Bench
identifica.

### 8.6 Solo dos de seis librerías se defienden del uso inseguro

De las seis librerías evaluadas, únicamente `nova-snark` **impide en
código** el setup de una sola parte en compilaciones de producción.
`risc0-zkvm` permite recibos sin garantía criptográfica pero ofrece una
opción explícita para bloquearlos, nombrando el escenario concreto —una
variable de entorno olvidada— que causa esos fallos en la práctica. Las
cuatro restantes lo permiten en silencio y confían en la documentación.

Es un eje que ninguna tabla de rendimiento captura, y distingue una
librería diseñada para producción de una diseñada para publicar
resultados.

### 8.7 El ecosistema PLONK/KZG en Rust está fragmentado verticalmente

Seis vías de implementación investigadas, cinco inviables: paquetes sin
publicar, funciones hash sin especificación para la curva requerida,
dependencias de repositorio sin fijar, y cadenas de dependencias que
exigen compiladores no estables.

### 8.8 Un zkVM no es comparable en igualdad de condiciones

Se evaluó RISC Zero como sexto paradigma. Su encaje conceptual era
favorable: emplea STARK sobre Goldilocks —el mismo sistema y campo que el
backend elegido— lo que habría permitido aislar una única variable: cómo
se expresa la lógica.

Sin embargo, requiere una cadena de herramientas externa para compilar el
programa invitado, lo que incumple el criterio metodológico aplicado a los
otros cinco (instalación mediante gestor de paquetes exclusivamente). No
es un defecto: es el precio de compilar programas arbitrarios.

La cifra que lo cuantifica: **3 dependencias frente a 349**.

---

## 8.bis La vía en dos fases y su encaje en ISO 20022

### El pagador necesitaba el saldo del receptor

Una liquidación que actualiza **las dos hojas** en una sola transición exige
que quien construye la prueba conozca **los dos saldos**. La
confidencialidad frente a terceros se mantiene; frente a la **contraparte**,
no: **pagar a alguien revela cuánto tiene**.

No es un defecto de implementación. Es una propiedad del enunciado que se
demuestra, y aparece en los cinco sistemas de prueba, porque una prueba sobre
dos cuentas exige conocer dos cuentas.

Importa más de lo que su tamaño sugiere: **el operador es una parte
declarada** cuyos poderes se cuentan y auditan; **una contraparte puede ser
cualquiera**.

### El diseño que lo cierra

| Fase | Qué ocurre | Qué saldo hace falta |
|---|---|---|
| `send` | El valor sale del pagador a un **pendiente** | Solo el del pagador |
| `claim` | El receptor lo hace suyo | Solo el del receptor |

> ⚠️ **Nota de corrección (cuarta revisión).** El circuito de cobro **no
> ataba el compromiso a la identidad de quien cobra** hasta el 30 de julio
> de 2026: cualquiera con el aviso podía reclamarlo. Las revisiones
> anteriores describieron el cobro como demostración de titularidad, que es
> lo que el diseño pretendía y lo que la implementación no imponía.
> Corregido y medido; ver `AUDITORIA.md` §27 y §39.1.

El compromiso liga la identidad pública del receptor, un aleatorio elegido
por el pagador y el importe. **Ninguna fase lee el saldo del otro**, y va en
la firma: no hay parámetro por donde entrara.

⚠️ **Coste declarado.** El pago no es firme hasta que se cobra; si el receptor
nunca cobra, el valor queda inmovilizado **hasta que el emisor lo reembolse**
(§178-§181; el plazo **no se cuenta en tiempo sino en entradas del registro**,
que hace avanzar el operador; y los pendientes anteriores a ese mecanismo son
irreembolsables); y el pagador, que eligió el aleatorio, puede recalcular el
compromiso y ver **cuándo** se cobra — no cuánto tiene, pero sí una señal
temporal que el diseño no elimina.

### Sin nullificador, y por qué

Los circuitos en dos fases **omiten el nullificador a propósito**: un envío
cambia el saldo, luego la hoja, luego la raíz de cuentas, así que un reenvío
parte de una raíz obsoleta y se rechaza.

⚠️ **Con consenso distribuido esto cambiaría**: el encadenamiento de raíces
exige un orden total, y el nullificador detecta un gasto repetido sin
necesitarlo.

### El encaje en ISO 20022

Un `pacs.008` produce un `pacs.002` con la prueba adjunta, y los rechazos
llevan códigos del catálogo `ExternalStatusReason1Code`, no cadenas propias.

El modelo en dos fases **usa el vocabulario del propio estándar**:

| Fase | Estado | Significado en el estándar |
|---|---|---|
| Envío aceptado | `ACSP` | Aceptada, liquidación en curso |
| Cobro aplicado | `ACSC` | Liquidación completada |
| Rechazo | `RJCT` | Con su código de motivo |

⚠️ **Falta una pieza.** El receptor necesita la posición del pendiente, el
aleatorio y el importe para cobrar, y **ISO 20022 no tiene campo para ellos**.
La implementación los devuelve junto al mensaje, no dentro. **Cómo viaja ese
canal lateral está sin resolver.**

---

## 9. Errores propios detectados y corregidos

Documentarlos es parte de la contribución metodológica: un trabajo sin
errores documentados suele indicar que la verificación fue superficial.

**Comparación entre compilaciones distintas.** Una versión preliminar de
las mediciones comparaba cifras de compilación de depuración con cifras
optimizadas, lo que hacía aparecer a STARK como 130× más rápido que
Groth16 cuando la relación real es ~11×.

**Nullifier privado.** Una versión inicial mantenía el nullifier como dato
privado, lo que impedía a la capa de estado mantener su árbol.

**Nullifier no insertado.** La operación de aplicación no insertaba el
nullifier en el árbol, lo que habría hecho vacua la garantía más costosa
del sistema.

**Restricciones vacías.** Durante la implementación del circuito de
congelación, dos restricciones quedaron escritas como marcadores
idénticamente nulos. Una restricción idénticamente cero **se satisface
siempre y no falla ningún test negativo**; fue detectada por revisión
manual, no automática.

**Tests no discriminantes.** En tres ocasiones un test negativo resultó
fallar por una restricción distinta de la que pretendía verificar. Se
corrigieron construyendo testigos internamente coherentes que violan
únicamente la restricción bajo prueba.

---

### Sustituir sin contrastar

Introducir la vía en dos fases y encaminar el puente ISO por ella **perdió en
silencio dos propiedades** que la vía original tenía:

1. **El límite regulatorio dejó de imponerse en el circuito.** El circuito
   antiguo lo lleva como entrada pública y demuestra `importe ≤ límite`; el
   nuevo no, así que el límite solo se comprobaba al generar — evitable
   construyendo la propia traza.
2. **Las operaciones no dejaban rastro en el registro.** El módulo de dos
   fases era **el único que no registraba nada**, y era ya la única vía
   institucional.

Las dos aparecieron al migrar los tests de la vía antigua y preguntar qué
defendía cada uno. Ninguna la encontraron los tests de la vía nueva, escritos
mirando solo la vía nueva.

> **Sustituir no es solo escribir lo nuevo. Es contrastar lo que hacía lo
> viejo.**

### Propiedades demostradas sobre un modelo que no se ejecuta

Un módulo prototipo llevaba ocho tests que demostraban las propiedades del
diseño. La producción usa **una función** de ese módulo y ninguna de sus
estructuras. Contrastar los ocho contra la vía ejecutada encontró una
propiedad de seguridad —que cobrar un importe distinto al comprometido se
rechaza— verificada **solo sobre el modelo**.

> **Una propiedad de seguridad demostrada sobre un modelo no está demostrada
> sobre lo que se ejecuta.**

### Tests de reinicio que comparan en vez de atacar

De doce tests de reinicio, **once comparaban un valor** antes y después; uno
intentaba la operación prohibida.

Convertir uno de los once encontró que el **máximo** del cupo de custodios no
se persistía mientras su **contador** sí: reiniciar el nodo renovaba un cupo
agotado, levantando cualquier restricción impuesta a un conjunto bajo
sospecha.

> **Comparar un valor restaurado es un indicio. Intentar la operación que
> debería bloquear es la propiedad.**

### Tests declarados frente a tests ejecutados

Un test escrito en el ámbito equivocado compilaba, no se registraba y **no
ejecutaba nada**. Solo lo delató contrastar los `#[test]` declarados con los
ejecutados.

> **Un test que no aparece en la lista es invisible: no falla y no avisa.**

### Dónde se encontraron

⚠️ Ninguno vino de las herramientas construidas para la auditoría —un
detector de restricciones vacías por mutación y un comprobador de columnas
sin rellenar—. La afirmación de versiones anteriores de este texto —que
entre las dos no encontraron ningún defecto en doce circuitos de
producción— era incorrecta en dos puntos, y la auditoría lo documenta: el
barrido de mutación cubría **once** circuitos, no doce —su informe sobre
`circuit_audit` trabajaba con una traza de referencia inválida, y la
autocomprobación que lo habría delatado es un `debug_assert` que nunca se
ejecutó porque toda la documentación indicaba `--release`—, y de esos
doce, dos pertenecen a una vía retirada del diseño, así que «de
producción» eran diez. En los once cubiertos, ninguna de las dos
herramientas encontró defectos.

Que la cifra publicada fuera 12 y la real 11 es, en sí, un dato del tipo
que este trabajo pretende aportar: **una herramienta de verificación cuya
autocomprobación nadie ha ejecutado no dice lo que parece decir.**

Todos vinieron de preguntar **qué defiende cada comprobación**, y después
intentar aquello que debería impedir.

---

## 10. Método de verificación

Cada propiedad de seguridad se verifica mediante un **test
discriminante**: un testigo internamente coherente que viola únicamente la
restricción concreta bajo prueba. Un testigo corrompido de forma
indiscriminada rompe varias restricciones simultáneamente, y el test pasa
aunque la restricción de interés no imponga nada.

Adicionalmente, varios tests incluyen **verificación del propio test**: se
comprueba que la prueba puede fallar. Por ejemplo, el test que verifica que
los saldos no son legibles en disco va acompañado de otro que verifica que
**sin cifrado sí lo son**; sin el segundo, el primero pasaría aunque la
búsqueda estuviera mal construida.

Esta disciplina detectó un caso concreto: un test de cifrado fallaba, y la
causa no era una fuga sino un valor de prueba que superaba el tope de
emisión, de modo que el estado nunca llegaba a crearse.

---

## 11. Ausencia de consenso: análisis

Esta sección analiza la limitación principal del trabajo. La incluimos
como sección propia, y no como nota al pie, porque delimita el alcance de
todas las garantías anteriores.

### 11.1 El operador y sus tres capacidades

La arquitectura implementada es de **nodo único**. Su operador dispone de
tres capacidades distintas:

| Capacidad | ¿Mitigada? |
|---|---|
| Observar todos los saldos | **No** |
| Ordenar operaciones y censurar | **No** |
| Reescribir el historial | **Sí** (§11.3) |

Las dos primeras son inherentes a la arquitectura: quien mantiene el
estado lo conoce, y quien procesa las operaciones controla su orden.
**Ninguna construcción criptográfica las elimina sin replicación del
estado entre partes que no confíen entre sí**, es decir, sin consenso.

### 11.2 Qué está y qué no está demostrado

La distinción es precisa y conviene formularla explícitamente:

| Afirmación | ¿Demostrada? |
|---|---|
| Esta transferencia conserva el dinero | Sí |
| El saldo de esta cuenta es X | Sí |
| Nadie gasta sin ser titular | Sí |
| Nadie gasta dos veces | Sí |
| **Este es el estado actual del sistema** | **No** |
| **Estas son todas las operaciones ocurridas** | **No** |

**Las transiciones de estado están demostradas. El estado y la completitud
del historial, no.**

### 11.3 Mitigación parcial: registro encadenado de transiciones

Sin abordar el consenso, sí es posible cerrar la tercera capacidad
mediante un registro encadenado.

Cada operación aplicada genera una entrada:

```
resumen_n = H(n, tipo, raíz_antigua, raíz_nueva, H(prueba), resumen_{n-1})
```

El encadenamiento hace que alterar una entrada antigua invalide todos los
resúmenes posteriores. **Publicar el resumen de cabeza —32 bytes—
compromete todo el historial**: dos copias con la misma cabeza tienen la
misma historia, y cualquier reescritura posterior las separa.

Es la construcción que emplea Certificate Transparency frente a las
autoridades de certificación: no impide el comportamiento incorrecto, lo
hace **detectable a posteriori**.

**Limitaciones de la mitigación**: nadie está obligado a observar; el
operador podría no publicar el registro; y la censura no deja rastro,
porque una operación nunca procesada no genera entrada y su ausencia es
indistinguible de que nunca se solicitó.

### 11.4 Una asimetría revelada por el registro

La construcción del registro reveló una propiedad no advertida
previamente: **la apertura de una cuenta es la única transición de estado
que no genera prueba**. No crea dinero —la cuenta nace con saldo cero—
pero sí modifica la raíz de estado.

En la implementación actual la operación genera una entrada de registro
con resumen de prueba nulo, lo que la hace explícitamente distinguible
para un verificador: sabe que esa transición está **registrada pero no
demostrada**.

---

## 12. Trabajo relacionado

**Sistemas desplegados.** Zcash implementa transacciones con privacidad
desde 2016 sobre un diseño más completo que el presentado aquí. Aztec,
Aleo y Miden desarrollan infraestructura con conocimiento cero con equipos
y horizontes temporales sustancialmente mayores.

**Evaluación comparativa.** zk-Bench proporciona evaluación de Groth16,
PLONK, halo2 y starky con metodología rigurosa. Nuestro trabajo se
diferencia en el objeto medido —aplicación completa frente a circuitos de
referencia— y en el tipo de resultado: hallazgos de diseño frente a
métricas de rendimiento.

**Autorización con conocimiento cero.** Existe trabajo reciente sobre
autorización con nullifiers evaluada sobre múltiples backends.

**Iniciativas institucionales.** Los programas de moneda digital de banco
central —Drex, mBridge, los pilotos del Eurosistema— abordan requisitos
similares con recursos incomparablemente mayores. Este trabajo no compite
con ellos; aporta mediciones sobre una cuestión concreta que esos
programas también deben resolver.

---

## 13. Conclusiones

Implementar la misma aplicación de liquidación en cinco paradigmas de
prueba revela diferencias que las comparaciones sobre circuitos de
referencia no capturan. La más significativa es que la aritmetización AIR,
al carecer de restricciones de copia, exige rediseñar las actualizaciones
de estado con un patrón específico, y que omitirlo produce un agujero de
solidez invisible para testigos honestos.

La decisión de paradigma resultó determinada por un criterio que no
aparece en las tablas de rendimiento: **la existencia o no de una
ceremonia de setup de confianza**. Para una aplicación de liquidación, esa
ceremonia constituye una dependencia permanente e inauditable cuya
compromisión permite crear dinero sin rastro detectable.

Las mediciones muestran que verificar cuesta entre el 0,5% y el 0,8% de
generar, asimetría que hace viable el modelo, y cuantifican su límite
principal: **126,2 MiB de pruebas acumuladas por cada mil transferencias**
—unidad corregida en la cuarta revisión: la cifra siempre fue binaria, y
se etiquetaba «MB»; en unidades SI son 132,3 MB—.

Los resultados delimitan también lo que no se ha demostrado. La
arquitectura de nodo único implica que las transiciones de estado están
demostradas pero el estado actual y la completitud del historial no lo
están, y cerrar esa brecha requiere consenso distribuido, que pertenece a
otra disciplina.

### Trabajo futuro

- **Consenso distribuido**, requisito para las garantías de §11.
- **Auditoría de seguridad externa**, no realizada.
- **Agregación recursiva o pruebas por lote**, para el límite de §7.3.
- **Verificación de firmas en circuito**, para autorización genuinamente
  separada (§5.3).

---

## 14. Disponibilidad y reproducibilidad

La implementación completa está disponible en:

**`https://github.com/USUARIO/REPOSITORIO`**

Requiere únicamente el compilador Rust estable; no emplea cadenas de
herramientas externas ni compiladores no estables.

```bash
# O de una vez, con los pines del canon comprobados:
bash tools/canon.sh --sello

cargo test -p zk-ssl --release              # capa: 269 tests (3 ignorados)
cargo test -p stark-experiment --release    # circuitos: 297 (10 ignorados)
cargo test -p zk-ssl-node --release         # nodo: 31
cargo test -p zk-ssl --release metrics -- --nocapture
```

El repositorio incluye un documento de preparación para auditoría con el
modelo de amenaza, la tabla de invariantes y su punto de imposición, y una
sección explícita con los aspectos donde los autores tienen menor
confianza.

---

## Agradecimientos

[A completar]

## Referencias

[A completar con las citas correspondientes a: Groth16, PLONK, Halo2,
STARK/FRI, Nova, Rescue-Prime, zk-Bench, Certificate Transparency, Zcash,
y la documentación de las librerías empleadas.]
