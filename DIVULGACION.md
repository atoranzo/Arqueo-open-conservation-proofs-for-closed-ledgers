# La ceremonia y el secreto

**Qué aprendí construyendo cinco veces el mismo sistema de pagos privado**

---

## El problema que nadie ha resuelto bien

Imagina que quieres transferir dinero y que se cumplan dos condiciones a
la vez.

**Primera**: nadie más puede ver cuánto tienes, cuánto envías ni a quién.
Ni el banco del vecino, ni un empleado curioso, ni quien pinche el cable.

**Segunda**: un supervisor debe poder comprobar que no estás blanqueando
dinero, que no superas los límites legales, y que ese dinero existía antes
de que lo enviaras.

Las dos parecen incompatibles. Si nadie puede ver nada, ¿cómo se
supervisa? Y si el supervisor lo ve todo, ¿dónde está la privacidad?

La solución que usamos hoy es sencilla y vieja: **confiar**. Un banco lo
ve todo y certifica que se cumplen las reglas. Funciona porque hemos
decidido que funcione, no porque haya nada que lo garantice.

Existe otra vía. Se llama **prueba de conocimiento cero**, y la idea es
casi contraintuitiva: demostrar que algo es cierto sin revelar por qué lo
es.

---

## Demostrar sin contar

El ejemplo clásico es una cueva con forma de anillo y una puerta cerrada
al fondo. Alguien dice conocer la palabra que abre la puerta.

En lugar de decírtela, entra por un lado y sale por el otro. Si conociera
la palabra, podría hacerlo. Si no, tendría que volver por donde entró y
solo acertaría la mitad de las veces. Repítelo veinte veces y la
probabilidad de que esté fingiendo es de una entre un millón.

**Has quedado convencido y sigues sin saber la palabra.**

Las pruebas de conocimiento cero hacen eso con matemáticas en lugar de
cuevas. Puedo demostrarte que mi saldo está entre diez mil y veinte mil
euros sin decirte cuál es. Puedo demostrarte que esta transferencia
conserva el dinero —que lo que sale de una cuenta entra en otra— sin
revelar las cuentas ni el importe.

No es teoría. Zcash lo usa desde 2016. Lo que no está resuelto es **cómo
aplicarlo a un sistema de liquidación completo**, con emisión de dinero,
supervisión, congelación de cuentas y todo lo demás.

Eso es lo que me puse a construir.

---

## La ceremonia

Aquí es donde la historia se pone rara.

La mayoría de los sistemas de pruebas de conocimiento cero necesitan unos
parámetros iniciales. Esos parámetros se generan a partir de un número
secreto. Y ese número **hay que destruirlo**.

Porque quien lo conserve puede **falsificar pruebas**. Podría demostrar
que tiene mil millones de euros cuando no tiene nada, y la prueba
verificaría perfectamente. Nadie lo detectaría. Nunca.

En la jerga se le llama **"basura tóxica"**.

Para evitar que una sola persona lo conozca, se organiza una **ceremonia
de setup**. Varias personas contribuyen cada una con su fragmento de
secreto, y basta con que **una sola** sea honesta y destruya el suyo para
que el conjunto sea seguro.

Las ceremonias reales son extraordinarias de leer. La de Zcash, en 2016,
tuvo participantes que compraron ordenadores nuevos con dinero en
efectivo, arrancaron desde discos que grabaron ellos mismos, hicieron los
cálculos sin conectarse jamás a internet, y luego **destruyeron
físicamente el disco duro con un martillo**, grabando el proceso en vídeo.

Uno de ellos lo hizo dentro de un coche circulando por una autopista, para
que nadie pudiera predecir su ubicación.

Es magnífico. Y también es exactamente el problema.

---

## Por qué eso no vale para un banco central

Piensa en lo que significa esa ceremonia si el sistema va a emitir la
moneda de un país.

**No puedes comprobarlo.** El vídeo del martillo demuestra que rompieron
un disco, no que no copiaran el secreto antes.

**No caduca.** Dentro de treinta años, esas personas —o quien tenga sus
archivos— seguirán teniendo la capacidad de falsificar pruebas si
guardaron algo.

**Y el fallo sería indetectable.** No hay auditoría posterior que lo
descubra, porque las pruebas falsas son matemáticamente indistinguibles de
las verdaderas.

Un banco central cuyo mandato es la soberanía monetaria estaría
delegándola permanentemente en un grupo de desconocidos que celebraron una
ceremonia hace décadas.

**No todos los sistemas necesitan ceremonia.** Y ahí empieza la parte
interesante.

---

## Cinco veces lo mismo

Hay al menos cinco familias de sistemas de pruebas de conocimiento cero, y
todo el mundo tiene opiniones sobre cuál es mejor. Las comparativas que
existen miden cosas pequeñas: cuánto tardan en demostrar que conocen la
entrada de una función hash, por ejemplo.

Mi sospecha era que esas comparativas **no capturan lo que importa**,
porque un sistema de pagos no es una función hash. Tiene estado que
persiste, invariantes que deben cumplirse siempre, y varias autoridades
con poderes distintos.

Así que hice algo tedioso: **implementé el mismo sistema de liquidación
cinco veces**, una en cada paradigma, y lo medí todo en las mismas
condiciones.

Meses de trabajo para responder a una pregunta que se enuncia en una
línea: *¿qué cambia de verdad?*

---

## La decisión que va contra los números

Los resultados fueron claros, y la elección obvia... hasta que dejó de
serlo.

**Groth16** es el más rápido y produce pruebas de **192 bytes**. Cabe en
un mensaje de texto.

**STARK** produce pruebas de **62 kilobytes**. Trescientas veces más
grandes.

Cualquier ingeniero sensato elegiría Groth16. Yo elegí STARK.

**Porque Groth16 necesita ceremonia y STARK no.** STARK se apoya
exclusivamente en funciones hash: no hay secreto que destruir, no hay
ceremonia que celebrar, no hay nadie en quien confiar.

Y hay un extra: como no usa las matemáticas que un ordenador cuántico
rompería, **sobrevive a la computación cuántica** sin rediseñarse.

Es la única decisión de todo el proyecto que tomé **contra** lo que decían
las mediciones. Y sigo pensando que es la correcta: en infraestructura
monetaria, doscientos bytes de más son un coste; una dependencia
inauditable y permanente es un fallo de diseño.

---

## El fallo que ningún test podía encontrar

Aquí está lo que hace que este trabajo aporte algo, y no es una medición.

Cada paradigma expresa el cómputo de forma distinta. Al portar el sistema
de uno a otro, encontré un agujero de seguridad **que solo existe en uno
de ellos y que es invisible**.

Sin tecnicismos: para actualizar el saldo de una cuenta hay que demostrar
dos cosas —cómo estaba antes y cómo queda después— y ambas demostraciones
deben referirse **a la misma cuenta**.

En algunos paradigmas puedes exigir directamente "estos dos cálculos usan
los mismos datos". En el paradigma que elegí, **eso no se puede
expresar**. Y si no lo notas, un atacante podría demostrar que modificó
una cuenta cuando en realidad modificó otra.

Lo inquietante no es el fallo. Es que **ningún test lo encuentra**.

Una implementación honesta usa siempre los mismos datos en ambos cálculos,
así que todas las pruebas legítimas funcionan y todas las pruebas
maliciosas que se te ocurran también fallan por otros motivos. El agujero
solo aparece si te sientas a preguntarte: *¿qué estoy exigiendo aquí, y
qué no?*

Ese hallazgo no habría salido midiendo funciones hash. Salió de construir
algo lo bastante complejo como para tener estado que actualizar.

---

## Lo que cuesta la privacidad, en números

Una parte del debate sobre monedas digitales de banco central discute si
la privacidad criptográfica es viable. Se dicen muchas cosas y se aportan
pocas cifras. Estas son las mías, medidas:

| | |
|---|---|
| Arrancar el sistema | **0,67 milisegundos** |
| Comprobar que una transferencia es válida | **4 milisegundos** |
| Generar la prueba de esa transferencia | **620 milisegundos** |
| Espacio de mil transferencias | **59 megabytes** |

Fíjate en la relación entre las dos del medio: **comprobar cuesta el medio
por ciento de demostrar**.

Eso es lo que hace viable todo el modelo. El trabajo pesado recae en quien
quiere probar algo; quien lo verifica apenas gasta nada. Un supervisor
podría comprobar millones de operaciones al día en un ordenador normal.

Y el último número es un problema: **59 megabytes por cada mil
transferencias**. Un sistema nacional haría millones al día.

Pero **no es el primero que aparecería**, y una versión anterior de este
texto decía que sí.

⚠️ **Y aquí hace falta corregir la corrección.** Durante un tiempo este
texto explicaba que el límite que primero aparecía era otro: cada pago
generaba una marca guardada en una casilla **calculada a partir de la
propia marca**, y por el mismo efecto que hace que en un aula de treinta
personas dos cumplan años el mismo día, a los **sesenta y cinco mil
pagos** la coincidencia ya era probable — dejando el pago de alguien
bloqueado para siempre.

**Ese límite ya no existe: el camino que lo producía se retiró entero**
(`AUDITORIA.md` §32 y §36), y hoy nada genera esas marcas. El pago se
protege ahora encadenando el estado: reenviar una operación vieja
presenta una foto caducada y se rechaza.

**Pero no se resolvió, se evitó** — y conviene decirlo así: el nuevo
mecanismo necesita que **alguien ponga las operaciones en un orden**, y
eso es justo lo que un sistema distribuido no tiene gratis. Quien
reparta esto entre varios nodos recupera el problema entero.

¿Y cuál es entonces el límite que primero muerde hoy? **No es el
tamaño**: es que cada prueba se ata a la foto exacta del sistema que vio
al generarse, así que dos personas pagando a la vez se estorban. Medido:
**entre 1,5 y 1,9 operaciones por segundo**. Ese es el techo real, y
está escrito con su número.

---

## Lo que no hace

Aquí es donde muchos proyectos se callan. Voy a hacer lo contrario, porque
creo que es lo que hace útil el trabajo.

**Esto no está descentralizado.** Funciona en un solo ordenador. Quien lo
opere ve todos los saldos y puede negarse a procesar operaciones.
Arreglarlo requiere consenso distribuido, que es un campo entero de la
informática que este proyecto no aborda.

**No lo ha auditado nadie.** Escribí más de doscientas pruebas que
intentan romperlo, y algunas lo consiguieron. Pero yo escribí el código y
yo escribí las pruebas, y esa es exactamente la ceguera que una auditoría
externa existe para corregir.

**Nunca ha movido dinero real.** No está en producción ni cerca.

**No es una blockchain.** No hay cadena de bloques, ni minería, ni tokens.

---

## Por qué publico mis errores

En la documentación del proyecto hay una sección con los fallos que
cometí. No los resúmenes elegantes: los fallos.

Comparé velocidades entre dos versiones distintas del programa, y durante
un tiempo creí que un sistema era ciento treinta veces más rápido que otro
cuando la diferencia real era once.

Escribí dos reglas de seguridad como marcadores vacíos, con intención de
completarlas después. Una regla vacía **se cumple siempre**: el sistema
parecía comprobar que solo dos autoridades pueden congelar una cuenta, y
no comprobaba nada.

Tres veces escribí pruebas que fallaban por el motivo equivocado, dándome
confianza injustificada.

Están todos publicados. Por dos razones.

La primera es práctica: un proyecto sin errores documentados suele
significar que **nadie miró de verdad**. Si alguien va a confiar en este
trabajo, merece saber cómo se verificó y dónde falló.

La segunda es que los errores son lo más instructivo que hay. Que una
restricción vacía no falle ningún test es una lección que no se olvida
cuando la sufres.

---

## Qué queda

Que sea infraestructura utilizable exige dos cosas que no dependen de
escribir más código.

**Consenso distribuido**, para que deje de haber un operador que lo ve
todo. Es un proyecto aparte, en otra disciplina, y hacerlo mal es peor que
no hacerlo: un consenso defectuoso da apariencia de garantía sin darla.

**Una auditoría externa**, que es la única condición de verdad. Sin
alguien de fuera intentando romperlo con conocimiento y mala intención,
nada de esto debería tocar dinero real.

---

## Lo que sí queda demostrado

Que las propiedades se pueden construir a la vez.

Privacidad frente a terceros. Cumplimiento verificable sin claves
maestras. Conservación del dinero, imposible de burlar. Supervisión que
funciona sin ver los saldos. Y todo ello **sin depender de ninguna
ceremonia ni de ningún secreto que alguien tuviera que destruir**.

No era evidente antes de construirlo. Ahora está medido, y cualquiera
puede reproducirlo en su ordenador con un solo comando.

Eso, y ocho hallazgos técnicos que no estaban documentados en ningún
sitio, es lo que aporta este trabajo. No una infraestructura lista para
usar: **datos sobre lo que cuesta construirla, y sobre lo que sale mal
mientras lo intentas**.

---

## Para mirarlo por dentro

**`https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers`**

Todo está publicado con licencia libre: el código, las mediciones, la
documentación técnica, los errores y un documento pensado para quien
quiera romperlo, que incluye una sección con los puntos donde tengo menos
confianza.

Se reproduce con Rust estable y un comando. Nada de lo que he contado aquí
requiere fiarse de mí.

---

*Angel Toranzo Portela · 2026*
*Licencia: MIT / Apache-2.0*
