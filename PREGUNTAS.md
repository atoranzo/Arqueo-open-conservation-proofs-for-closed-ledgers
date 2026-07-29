# ZK-SSL — 22 preguntas

Las preguntas que hace alguien que se encuentra con este proyecto,
respondidas sin adornos. Si una respuesta te parece incómoda, es que está
bien escrita.

---

## QUÉ

### 1. ¿Qué es esto exactamente?

Dos cosas.

Una **capa de liquidación financiera** donde las transferencias son
privadas y el cumplimiento normativo es demostrable criptográficamente,
construida sin ninguna ceremonia de confianza.

Y el **trabajo comparativo** que fundamentó su diseño: el mismo circuito
implementado en cinco sistemas de prueba de conocimiento cero y medido en
condiciones idénticas.

De las dos, la segunda es la que aporta algo que no existía.

### 2. ¿Qué NO es?

**No es una blockchain.** Es un nodo único.

**No es descentralizado.** Quien lo opera ve todos los saldos y puede
censurar operaciones.

**No está auditado.** Nadie externo lo ha revisado.

**No está en producción** ni lo ha usado nadie con dinero real.

### 3. ¿Qué garantiza?

Sin revelar identidades, saldos ni importes: que nadie puede crear dinero,
gastar de una cuenta ajena, gastar dos veces, gastar estando congelado,
reenviar una operación válida ni operar sobre un estado corrupto.

Cada una de esas garantías tiene un test que intenta romperla.

### 4. ¿Qué NO garantiza?

Que el estado que muestra el operador sea el real, y que las operaciones
registradas sean todas las que ocurrieron.

**Las transiciones están demostradas. El estado y la completitud del
historial, no.** Eso exige consenso distribuido.

### 5. ¿Qué aporta que no existiera?

Ocho hallazgos que no están en la literatura comparativa, porque solo
aparecen al portar una **aplicación completa** entre paradigmas, no un
SHA-256 de referencia.

El principal: la aritmetización AIR **carece de restricciones de copia**,
lo que abre un agujero de solidez silencioso al actualizar árboles de
Merkle. Invisible para testigos honestos.

---

## POR QUÉ

### 6. ¿Por qué STARK y no Groth16, que es más rápido?

Porque Groth16 exige una **ceremonia de confianza**. Si sus participantes
coluden y conservan el secreto, pueden falsificar pruebas — y en una capa
de liquidación eso significa **crear dinero sin dejar rastro**. Las pruebas
falsas verifican correctamente.

El precio de evitarlo: pruebas de 62 KB en vez de 192 bytes. Un factor de
320.

Es la única decisión del proyecto tomada **contra** los números de
rendimiento.

### 7. ¿Por qué no hay consenso?

Porque es un problema de sistemas distribuidos, no de criptografía, y
requiere entre 20 y 40 rondas de trabajo en un campo donde nada de lo
aprendido aquí ayuda.

Y hay una razón más fuerte: **un consenso mal implementado es más
peligroso que ninguno**, porque da apariencia de garantía sin darla. Los
fallos bizantinos son sutiles y no se detectan con tests.

### 8. ¿Por qué se documentan los errores propios?

Porque un trabajo sin errores documentados suele significar que nadie miró
de verdad.

Están registrados los tres tests que no discriminaban, la comparativa que
mezcló compilaciones de depuración con optimizadas, y las dos restricciones
que quedaron escritas como marcadores vacíos —que se satisfacen siempre y
no fallan ningún test negativo—.

### 9. ¿Por qué el operador sigue viendo los saldos?

Porque mantiene el estado. Quien guarda el árbol de cuentas conoce su
contenido.

La privacidad de este sistema es **frente a terceros que solo ven
pruebas**, no frente a quien mantiene el ledger. Eliminarlo exige replicar
el estado entre partes que no confíen entre sí — es decir, consenso.

### 10. ¿Por qué una cuenta congelada puede seguir recibiendo?

Porque impedirlo dejaría fondos en el limbo y rompería pagos legítimos
hacia una cuenta bajo investigación.

Un pagador honesto no sabe que el destinatario está congelado. Rechazar el
pago le perjudica a él, no al investigado.

### 11. ¿Por qué la revelación selectiva depende del titular?

Porque la alternativa sería una clave maestra de supervisión, y esa clave
es un objetivo. **Aquí no hay ninguna clave que robar** para obtener acceso
general a los saldos.

La contrapartida está declarada: si el titular se niega a cooperar, no hay
mecanismo de revelación forzosa.

---

## CÓMO

### 12. ¿Cómo se demuestra que no se crea dinero?

Con partida doble dentro del circuito: lo que sale de una cuenta entra en
otra, y ambas subidas del árbol de Merkle están atadas a la misma
posición.

El suministro total es **público** y solo cambia mediante emisiones o
destrucciones demostradas, cada una con su prueba.

### 13. ¿Cómo se impide el doble gasto?

Cada operación genera un **nullifier** derivado de la clave de gasto y del
nonce. El circuito demuestra que su posición en el árbol estaba **libre**
antes de la operación, y la capa lo inserta al aplicarla.

Solo el titular puede calcular su nullifier, lo que impide a un observador
precomputarlos para vigilar cuándo gasta una cuenta ajena.

### 14. ¿Cómo funciona la supervisión?

El titular genera una prueba de que su saldo está en un rango. Tres modos
con el mismo circuito:

| Modo | Revela |
|---|---|
| Exacto | El saldo |
| Mínimo | Que supera X |
| Banda | Que está entre X e Y |

El supervisor la verifica con una función libre, **sin acceso al ledger**.

### 15. ¿Cómo se evita que un custodio comprometido emita solo?

La emisión exige dos custodios distintos. El riesgo real no es que firme
alguien de fuera —eso lo cierra la pertenencia al conjunto— sino que **el
mismo custodio cuente como dos**, lo que convertiría un 2-de-N en un
1-de-N encubierto.

Se cierra con índices estrictamente crecientes **atados a los caminos de
Merkle** mediante un acumulador. Sin esa segunda parte, el índice sería un
número declarado sin relación con la posición demostrada.

### 16. ¿Cómo sé que el operador no ha reescrito el historial?

Por el **registro encadenado de transiciones**. Cada operación deja una
entrada cuyo resumen incluye el de la anterior, así que alterar el pasado
invalida todo lo posterior.

**Publicar la cabeza —32 bytes— compromete todo el historial**: dos copias
con la misma cabeza tienen la misma historia.

Es lo que hace *Certificate Transparency* con las autoridades de
certificación: no impide que se porten mal, hace que no puedan hacerlo en
secreto.

---

## QUIÉN

### 17. ¿Quién puede crear dinero?

Dos custodios distintos de un conjunto comprometido en una raíz pública, y
solo hasta un **tope inmutable** del ledger.

Ni siquiera el conjunto completo puede superar ese tope sin crear un
ledger nuevo, lo que dejaría un rastro imposible de ocultar.

### 18. ¿Quién controla a los custodios?

Un **conjunto de gobernanza** distinto, que puede cambiar el de custodios.

La circularidad no desaparece —quien controle la gobernanza controla
todo— pero se traslada a claves que se usan casi nunca y pueden guardarse
sin conexión, frente a claves operativas expuestas a diario.

**El conjunto de gobernanza es inmutable.** Si se compromete, la única
salida es crear un ledger nuevo. Es el final consciente de la cadena de
autoridad.

### 19. ¿A quién le sirve esto?

Sinceramente: **a quien quiera datos sobre cómo eligen paradigma los
sistemas ZK**, y a quien esté evaluando si la privacidad con cumplimiento
demostrable es viable y a qué coste.

No a quien busque infraestructura desplegable. Para eso están Zcash o
Aztec, con años y equipos de ventaja.

---

## CUÁNTO

### 20. ¿Cuánto cuesta operar?

| | |
|---|---|
| Arrancar la capa | **0,67 ms** |
| Verificar una transferencia | ~4 ms |
| Generarla | ~620 ms |
| **Verificar / generar** | **0,5 %** |

> ⚠️ **Esa razón es la de la AUDITORÍA, no la de la transferencia.**
>
> `verify_audit` **solo verifica**: 1,6 ms frente a 274 de generación, un
> **0,58 %**. Es la cifra correcta para el argumento que sostiene —un
> supervisor comprueba sin tocar el estado— pero **estaba atribuida a la
> transferencia**.
>
> Aplicar una transferencia cuesta **17,5 %** de generarla, porque `apply`
> **verifica, muta el árbol y escribe a disco**. No es comparable.
>
> Se detectó ejecutando `cargo test -p zk-ssl --release metrics --
> --nocapture` y comparando con lo publicado. Ver `AUDITORIA.md` §22.

Esa asimetría es lo que hace viable el modelo: el coste recae en quien
produce la prueba, no en quien la acepta.

⚠️ Una sola ejecución en una máquina. Sirve para comparar órdenes de
magnitud, no como benchmark.

### 21. ¿Cuánto ocupa? ¿Escala?

**62 KB por transferencia. Mil transferencias son 59,1 MB acumulados.**

Resolverlo exige agregación recursiva o pruebas por lote, que no están
implementadas.

⚠️ **Pero no es el límite que primero muerde**, y una versión anterior de
esta respuesta decía que sí.

La posición de un nullifier **se deriva del propio nullifier**, y el
circuito exige que esté libre. Por la paradoja del cumpleaños, a los
**~65.000 pagos** la probabilidad de que dos caigan en la misma posición
ya es del 39 %.

Y el afectado **no puede reintentar**: su nullifier es determinista, así
que su pago queda bloqueado de forma permanente.

**Los 59,1 MB son un coste. La colisión es una parada**, y le ocurre a un
usuario concreto sin que el sistema esté saturado.

Hay dos límites más —el árbol de pendientes se agota a los 2³² pagos
totales, y el conjunto de custodios tope en 128— y los cuatro están en
`AUDITORIA.md` §13.

### 22. ¿Cuánto falta para que sea usable?

Para uso real: **consenso distribuido** y **auditoría externa**. La primera
es un proyecto aparte; la segunda no depende de más código.

Y hay algo que conviene decir: **el objetivo de este trabajo nunca fue
llegar a producción**. Fue averiguar qué cambia al implementar lo mismo en
cinco paradigmas, y construir algo lo bastante completo como para que la
respuesta significara algo.

Eso está conseguido y medido.

---

## Para seguir

| | |
|---|---|
| Empezar | [`README.md`](./README.md) |
| Romperlo | [`AUDITORIA.md`](./AUDITORIA.md) |
| La comparativa | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| El artículo | [`PAPER.md`](./PAPER.md) |
