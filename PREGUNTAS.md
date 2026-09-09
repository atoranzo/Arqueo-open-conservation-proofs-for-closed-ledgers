# Arqueo — 22 preguntas

Las preguntas que hace alguien que se encuentra con este proyecto, respondidas sin adornos. Si una
respuesta te parece incómoda, es que está bien escrita. Cada afirmación en presente apunta al
fichero o al asiento de [`AUDITORIA.md`](./AUDITORIA.md) que la sostiene; lo que está verificado
lo está contra `main` en el commit `9c64fd1`. En inglés: [`QUESTIONS.md`](./QUESTIONS.md).

---

## QUÉ

### 1. ¿Qué es esto exactamente?

Un **motor de libro cerrado que publica pruebas abiertas**. Un operador lleva un libro —cuentas,
pagos, emisiones, retiradas—; quienes dependen de él no pueden verlo. Arqueo hace que el libro
publique, cada época, una **cabeza firmada** y **paquetes de evidencia** con los que un tercero
comprueba, sin el libro, sin red y sin fiarse del autor, que el libro hizo lo que sus reglas dicen:
que el dinero se conserva, que una etiqueta se consumió una sola vez, que la historia no se
reescribió, que una entrada está dentro, que sólo el titular movió su cuenta.

Debajo hay una capa de liquidación en Rust con pagos en dos fases probados con STARK, y el trabajo
comparativo que fundamentó su diseño: el mismo circuito en cinco sistemas de prueba
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)). Antes se llamaba ZK-SSL; cambió el nombre del
proyecto, no los identificadores publicados (`zkssl/0.3`, `zk-ssl-*`).

### 2. ¿Qué NO es?

**No es una cadena.** Un nodo, un escritor. **No es descentralizado**: quien lo opera ve todos los
saldos y puede omitir una operación sin dejar rastro. **No está auditado** por nadie externo. **No
está en producción** ni lo ha usado nadie con dinero real. **No prueba solvencia**: prueba que el
libro es coherente consigo mismo, no que sus unidades existan fuera de él (el límite oráculo,
[`SECURITY.md`](./SECURITY.md)). Y no es dinero cuántico: es la aproximación clásica, con un
intermediario mínimo y medido.

### 3. ¿Qué garantiza?

A un tercero que no ve el libro, hoy, medido: **conservación** (suministro = saldos + en vuelo,
también al reabrir el libro: `AUDITORIA.md` §379, §387–§394); **uso único** de una etiqueta dentro
de un libro, publicado en su cabeza firmada, y **detección** de la misma etiqueta en dos libros
([`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md)); **historia no
reescribible** con prueba de extensión (`zkssl_consistencyProof`); **inclusión con recibo**
(`zkssl_inclusionReceipt`, `zkssl_ackPath`); **autoría sin que la clave viaje**. La tabla, con su
fuente por fila, está en [`doc/USE_CASES.md`](./doc/USE_CASES.md).

Y dentro del libro, en circuito: nadie crea dinero, nadie gasta de una cuenta ajena, nadie gasta
dos veces, una cuenta congelada no gasta, una operación válida no se reenvía, y no se opera sobre
un estado corrupto. Cada una de esas garantías tiene un test que intenta romperla.

### 4. ¿Qué NO garantiza?

Que una operación omitida se detecte: la censura no deja rastro. Que el operador no vea los saldos:
los ve. Que dos libros no acepten la misma etiqueta: pueden; lo que hay es detección, después,
con las dos cabezas firmadas. Quién está detrás de una clave, ni que una persona tenga una sola
cuenta. Que un pago sea firme antes de cobrarse: hasta el cobro no lo es, y si nadie cobra, el
importe queda inmovilizado hasta que el emisor lo reembolse (`AUDITORIA.md` §178–§181). Y dos
propiedades que el motor **quiere** responder y todavía no responde: corte y completitud, y rechazo
con causa ([`doc/USE_CASES.md`](./doc/USE_CASES.md), filas 6 y 7).

### 5. ¿Qué aporta que no existiera?

Dos cosas. La primera, ocho hallazgos que no están en la literatura comparativa porque sólo aparecen
al portar una **aplicación completa** entre paradigmas, no un SHA-256 de referencia; el principal:
la aritmetización AIR **carece de restricciones de copia**, lo que abre un agujero de solidez
silencioso al actualizar árboles de Merkle ([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)).

La segunda, un **kit** con el que un tercero comprueba en su máquina, sin red y sin el repositorio,
un expediente que cuadra, uno manipulado que no cuadra —con la regla rota nombrada—, la misma
etiqueta publicada en dos libros, y un intercambio de libros rechazado con su nombre
([`doc/KIT.md`](./doc/KIT.md)). No es una demo: son capturas reales de nodos que después se
apagaron, y el binario se reproduce desde el commit que su `VERSION` nombra.

---

## POR QUÉ

### 6. ¿Por qué STARK y no Groth16, que es más rápido?

Porque Groth16 exige una **ceremonia de confianza**. Si sus participantes coluden y conservan el
secreto, pueden falsificar pruebas — y en un libro eso significa **crear dinero sin dejar rastro**.
Las pruebas falsas verifican correctamente.

El precio de evitarlo: pruebas de unos 62 KB en vez de 192 bytes, un factor de 320
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)). Es la única decisión del proyecto tomada **contra** los
números de rendimiento.

### 7. ¿Por qué no hay consenso?

Porque es un problema de sistemas distribuidos, no de criptografía, y **un consenso mal implementado
es más peligroso que ninguno**: da apariencia de garantía sin darla.

El camino que este proyecto sí toma es el de *Certificate Transparency*: no impedir que el operador
se porte mal, sino que **no pueda hacerlo en secreto**. Para eso existen el firmante de cabezas
(`AUDITORIA.md` §236), el guardián del índice de firma (§234), el latido (§241), el verificador
independiente (§243) y los testigos que fijan la clave la primera vez que la ven (§245). Lo que le
falta: un ancla anterior al primer encuentro y una custodia de clave **comprobada**, no sólo
declarada (§244).

### 8. ¿Por qué se documentan los errores propios?

Porque un trabajo sin errores documentados suele significar que nadie miró de verdad.

Están registrados los tests que no discriminaban, la comparativa que mezcló compilaciones de
depuración con optimizadas, las restricciones que quedaron escritas como marcadores vacíos, la
cifra de rendimiento atribuida al nodo que medía otra cosa, y las frases de estos documentos que
envejecieron mientras el árbol avanzaba. La regla es una: **una cifra publicada que se corrige no
se borra, se marca** (`AUDITORIA.md` §247), y los depósitos con DOI tienen su fe de erratas en
[`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

### 9. ¿Por qué el operador sigue viendo los saldos?

Porque mantiene el estado: quien guarda el árbol de cuentas conoce su contenido. Eliminarlo exige
replicar el estado entre partes que no confíen entre sí, es decir, consenso.

Y una corrección que este documento debe llevar: durante un tiempo dijo que la privacidad era
«frente a terceros que sólo ven pruebas». **Era falso y está medido** (`AUDITORIA.md` §93): el
camino de Merkle que el protocolo entrega llevaba la hoja del vecino, y un diccionario recuperaba
un saldo. Desde §156 la hoja va envuelta con un salt y ese diccionario ya no acierta; desde §157
los índices no son enumerables ni predecibles. Lo que queda abierto está enumerado en
[`SECURITY.md`](./SECURITY.md). Lo que Arqueo ofrece a un tercero **no es privacidad: son pruebas**.

### 10. ¿Por qué una cuenta congelada puede seguir recibiendo?

Porque impedirlo dejaría fondos en el limbo y rompería pagos legítimos hacia una cuenta bajo
investigación. Un pagador honesto no sabe que el destinatario está congelado; rechazar el pago le
perjudica a él, no al investigado. Lo que una cuenta congelada no puede hacer es **gastar**: la
no-pertenencia al árbol de congelados se demuestra en circuito, y ese árbol tiene raíz en reposo
desde §391.

### 11. ¿Por qué la revelación selectiva depende del titular?

Porque la alternativa sería una clave maestra de supervisión, y esa clave es un objetivo. **Aquí no
hay ninguna clave que robar** para obtener acceso general a los saldos. La contrapartida está
declarada: si el titular se niega a cooperar, no hay mecanismo de revelación forzosa.

---

## CÓMO

### 12. ¿Cómo se demuestra que no se crea dinero?

Con partida doble dentro del circuito: lo que sale de una cuenta entra en otra, y ambas subidas del
árbol de Merkle están atadas a la misma posición. El suministro total es **público** y sólo cambia
mediante emisiones o destrucciones demostradas, cada una con su prueba.

Y desde §379 el invariante **suministro = saldos + pendientes** se comprueba al abrir el libro,
también al reabrirlo tras un reinicio (§387–§394), con un test que lo falsifica: si un byte del
estado en reposo crea dinero, el libro no abre.

### 13. ¿Cómo se impide el doble gasto?

Dentro de un libro, por el **encadenamiento de raíces**: cada prueba se ata a la raíz exacta que vio
al generarse, y el nodo único da el orden total que lo hace valer. Es anti-replay por construcción,
y es también el límite que primero muerde (pregunta 21).

Además, desde §413, un libro **publica lo que consume**: una etiqueta `H(dominio, identificador
acordado)` entra una sola vez en su árbol de consumos, cuya raíz va en la cabeza firmada; el sobre
de consumo demuestra, con dos cabezas del mismo libro, la ausencia bajo la vieja y la presencia
bajo la nueva. Entre libros distintos no hay orden que imponer: dos libros pueden aceptar la misma
etiqueta, y un tercero con las dos cabezas firmadas **lo ve, después**
([`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md)).

La vía antigua, que derivaba la posición de un *nullifier* del propio *nullifier*, **se retiró con
su árbol** (`AUDITORIA.md` §32 y §36): hoy nada los genera.

### 14. ¿Cómo funciona la supervisión?

Sin abrir el libro, por dos vías. La primera, del **titular**: una prueba de que su saldo está en un
rango, con tres modos del mismo circuito —exacto, mínimo, banda— que el supervisor verifica con una
función libre, sin acceso al libro. La segunda, del **libro**: la cabeza firmada y el paquete de
evidencia, que un supervisor comprueba con el verificador independiente, apagado el nodo. Lo que se
comprueba ahí es la pregunta 3; lo que no, la 4.

### 15. ¿Cómo se evita que un custodio comprometido emita solo?

La emisión exige dos custodios distintos. El riesgo real no es que firme alguien de fuera —eso lo
cierra la pertenencia al conjunto— sino que **el mismo custodio cuente como dos**, lo que
convertiría un 2-de-N en un 1-de-N encubierto. Se cierra con índices estrictamente crecientes
**atados a los caminos de Merkle** mediante un acumulador; sin esa segunda parte el índice sería un
número declarado sin relación con la posición demostrada.

### 16. ¿Cómo sé que el operador no ha reescrito el historial?

Por el **registro encadenado de transiciones**: cada operación deja una entrada cuyo resumen incluye
el de la anterior, y publicar la cabeza compromete todo el historial. Pero un registro encadenado
sólo delata reescrituras a quien ya vio una cabeza anterior. Por eso el nodo sirve la cabeza
**firmada** (`zkssl_signedEpochHead`), la firma es de la familia de las basadas en hashes (XMSS,
§236), un **testigo** independiente la verifica, la **cofirma** y fija la clave que ve la primera
vez (§245), y cualquiera puede pedir la prueba de que la cabeza de hoy **extiende** la de ayer
(`zkssl_consistencyProof`).

**La garantía la tiene quien mira, no quien lee**: sin un testigo corriendo, la frase de arriba no
protege a nadie.

### 17. ¿Cómo compruebo un expediente sin el nodo, sin red y sin fiarme del autor?

Con el kit ([`doc/KIT.md`](./doc/KIT.md)): una descarga, la release `arqueo-verify-v0.2.0`, cuya
huella se publica con su commit al lado. El verificador es una CLI de una línea:
`./zk-ssl-verify <fichero.json>` sale 0 y escribe `VERDE: …` si el fichero se sostiene, 1 y el
**primer** fallo con nombre (`ROJO: …`) si no. Cuatro pasos: un expediente que cuadra; uno
manipulado que no cuadra, con la regla rota nombrada; la misma etiqueta en dos libros, detectada
con las dos cabezas y los dos nodos apagados; y un intercambio de libros, rechazado con su nombre.
Los catálogos enteros se comprueban con el arnés que viaja dentro del tarball, y el binario se
reproduce desde el commit que su `VERSION` nombra.

---

## QUIÉN

### 18. ¿Quién puede crear dinero?

Dos custodios distintos de un conjunto comprometido en una raíz pública, y sólo hasta un **tope
inmutable** del libro. Ni siquiera el conjunto completo puede superar ese tope sin crear un libro
nuevo, lo que dejaría un rastro imposible de ocultar. Y lo que emiten se cuenta: el suministro es
público y el invariante de la pregunta 12 lo vigila.

### 19. ¿Quién controla a los custodios?

Un **conjunto de gobernanza** distinto, que puede cambiar el de custodios; cada cambio queda contado
en el registro (`governance_change_count`, §393). La circularidad no desaparece —quien controle la
gobernanza controla todo— pero se traslada a claves que se usan casi nunca y pueden guardarse sin
conexión, frente a claves operativas expuestas a diario. Si la gobernanza se compromete, la salida
es un libro nuevo: es el final consciente de la cadena de autoridad.

### 20. ¿A quién le sirve esto?

A quien lleva un libro cerrado del que dependen terceros que no pueden verlo, cuando la unidad
**nace y muere dentro del libro**: sistemas de depósito y retorno, garantías de origen y derechos de
emisión, monedas comunitarias, custodia de fondos de clientes, ayudas públicas donde el fraude es
la doble financiación, registros de derechos y cupos, compensación entre operadores o entre
administraciones. Los casos, con qué propiedad resuelve cada uno y cuáles están revisados, están en
[`doc/USE_CASES.md`](./doc/USE_CASES.md). No sirve a una entidad de contrapartida central —su
problema es el riesgo de contraparte, no la conservación— ni es un componente de una moneda digital
de banco central.

Sinceramente: hoy nadie lo usa con dinero real. Lo que hay es un motor medido, un formato con
vectores y un kit que invita a comprobarlo. Y el trabajo comparativo sigue sirviendo a quien quiera
datos sobre cómo eligen paradigma los sistemas de conocimiento cero.

---

## CUÁNTO

### 21. ¿Cuánto cuesta, cuánto ocupa, hasta dónde escala?

Todo medido en una máquina y en release; los tiempos van como rango porque dos tandas del mismo
binario difieren un ~9 % (`AUDITORIA.md` §131), y **no son comparables con medidas de otra sesión**.

- **Verificar frente a generar**: verificar una prueba de auditoría cuesta el 0,58 % de generarla
  (§22); esa asimetría es lo que hace viable el modelo. Aplicar una transferencia no es comparable:
  verifica, muta el árbol y escribe a disco.
- **Techo del nodo por RPC**: 248 operaciones por segundo (§229). El ciclo entero de un pago en un
  portátil —generar las pruebas de las dos partes y aplicarlas— sale a 1,5-1,9 pagos por segundo,
  y durante un tiempo esa cifra se atribuyó al nodo: **era falso, y por mucho** (§229, §238); el
  nodo trabaja el 4 % de ese ciclo.
- **Tamaño**: mil transferencias son ~590 s de prueba y 126,2 MiB acumulados (§130). Resolverlo
  exige agregación recursiva o pruebas por lote, que no están implementadas.
- **El límite que primero muerde**: la contención del anclaje de raíz. Cada prueba se ata a la raíz
  exacta que vio, así que dos emisores concurrentes se serializan; con cuatro clientes a la vez,
  uno aplica y los otros tres se rechazan (§123, §230). Con un solo emisor por raíz el desperdicio
  es cero.
- **Otros dos**: el árbol de pendientes se agota a los 2³² pagos simultáneos en vuelo, y el conjunto
  de custodios tope en 128 (§13). El límite de la colisión de *nullifiers* de la vía antigua
  **no se resolvió, se evitó**: la vía se retiró (§32, §36), y el encadenamiento que la sustituye
  exige un orden total que un nodo único da y un sistema distribuido no.

### 22. ¿Cuánto falta para que sea usable?

Para que un tercero real se apoye en estas pruebas: una **auditoría externa**, que no depende de
más código; una **custodia de clave comprobada**, no sólo declarada (§244); y un **ancla anterior
al primer encuentro** del testigo con el nodo. Para que el motor responda a todo lo que quiere
responder: el corte y la completitud, y el rechazo con causa, que existen como filas planeadas y no
como código. El consenso distribuido es otra disciplina y no es el camino de este proyecto: el
camino es la responsabilidad demostrable, y sus piezas están construidas (pregunta 7).

Lo que ya está: el formato como contrato público con vectores que nunca se reescriben, el
verificador reproducible, y un kit con el que cualquiera puede comprobar lo anterior sin creerse
esta página.

---

## Para seguir

| | |
|---|---|
| Empezar | [`README.md`](./README.md) |
| Comprobarlo sin fiarte de nadie | [`doc/KIT.md`](./doc/KIT.md) |
| Dónde encaja y dónde no | [`doc/USE_CASES.md`](./doc/USE_CASES.md) |
| Lo que sigue abierto | [`SECURITY.md`](./SECURITY.md) |
| Romperlo | [`AUDITORIA.md`](./AUDITORIA.md) |
| La comparativa | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| El artículo | [`PAPER.md`](./PAPER.md) |
