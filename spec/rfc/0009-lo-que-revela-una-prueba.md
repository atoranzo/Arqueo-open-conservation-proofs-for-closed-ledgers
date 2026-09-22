# RFC-0009 — Lo que revela una prueba: la promesa mientras el probador no oculte su testigo

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 162, 163, 164, 165 y 166)
- **Fecha:** 2026-09-21
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). Este RFC no
  cambia un método, un tipo del cable ni un vector: cambia lo que se promete de ellos.
- **Asiento(s) de AUDITORIA:** §521 (el testigo se publica), §522 (los comentarios y el literal del
  API), §523 (el modelo de las columnas constantes), §524 (el PASTE-367-M3, y lo que era de
  Groth16), §525 (los comentarios, con el censo que dice de qué sistema habla cada frase) y §526
  (el PASTE-367-M4); el §527, que lo adopta; y el §528, que le añade la E3.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: qué se promete (D-A), lo que sale literal en cada prueba (D-B), el principio del API como regla que hoy no se cumple (D-C) y la ocultación fuera de este RFC (D-D) | no | sellada en el §527 |
| E2 — el testigo de la tabla | una suite que produce cada tipo de prueba y cuenta sus valores literales contra la tabla de D-B, con un control que tiene que dar cero (D-E) | no | por hacer |
| E3a — el probador que oculta, dentro y apagado | el fork de winterfell 0.13.1 en el árbol, con la ocultación entera en el núcleo y sin tocar un AIR (D-F a D-J); apagado, cada prueba sale byte a byte como la de winterfell; y los falsadores de los spikes como tests del árbol, con el modo oculto solo en los tests (D-K) | no | por hacer |
| E3b — encenderlo | las pruebas que cruzan el cable salen ocultas; la tabla de D-B pasa a cero, con la suite de E2 como testigo (D-K) | sí: `zkssl/0.4` | por hacer; espera a E2 y a E3a |

Las medidas de este documento son las de los asientos §521, §523, §524 y §526: lecturas puras que
restauraron el árbol con sha y porcelain. Sus instrumentos viven fuera del árbol, en Downloads del
autor, como los de los RFC anteriores.

## Motivación

Las pruebas STARK de la casa se producen con winterfell 0.13, y winterfell no oculta el testigo: su
portada pone el conocimiento cero perfecto entre lo que espera añadir, no entre lo que ofrece, y su
issue 9 lo dice sin rodeos. Lo midieron cuatro lecturas puras en las sesiones 162 y 163 (§521): cada
prueba abre 42 filas —42 es el número de consultas de `ProofOptions`— y lo que esas filas llevan se
publica. La clave de gasto sale 42 veces en SEND-v1, SEND-v2, CLAIM y PRENDA.

La sesión 163 dio con la regla que lo explica y la sometió a dos falsadores que podían tumbarla
(PASTE-360-M2, §523): **un valor sale LITERAL en una prueba si y sólo si su columna es CONSTANTE en
la traza**. Lo que se abre no son filas de la traza sino de su extensión, evaluada en un coset
desplazado, y allí sólo una constante vale lo mismo que en la traza. Con esa regla, las sesiones
163 y 164 midieron todo lo que es STARK (§523, §524 y §526). El camino de merkle, que no lleva nada
constante, dio cero (§524).

La SOLIDEZ no cae: cada prueba sigue probando lo que dice. Cae la OCULTACIÓN, y con ella la
custodia de la clave frente a quien vea una prueba —el nodo las recibe todas— y la privacidad
frente a quien lea un sobre publicado. Los cortes §521 a §526 dejaron de afirmar lo contrario en la
prosa y en los comentarios. Falta decir, en un sitio normativo, qué se promete: eso es este RFC.

## Diseño

Las cinco decisiones las tomó el asistente por delegación del autor (sesión 165), con la
constitución de decisión (pureza, claridad, coherencia, imagen fiel, en ese orden). Todas llevan
su condición de reversión, escrita aquí.

### D-A — Se promete la solidez, no la ocultación

Una prueba de Arqueo que verifica demuestra la transición que enuncia. **No oculta su testigo.** La
regla que se promete es la medida, con sus palabras: lo que va en una columna CONSTANTE de la traza
sale LITERAL en la prueba; lo que no es constante no sale literal, pero nada garantiza que no se
deduzca de lo que se abre, porque el probador no enmascara. «No prueba» no es «no revela», y «no
sale literal» no es «oculto».

**Confianza residual, en una frase:** quien reciba una prueba —y el nodo las recibe todas— ve lo que
dice la tabla de D-B, y nada de este documento le impide deducir más.

Dos caminos se pesaron: (a) prometer sólo la solidez y tabular lo que sale; (b) prometer lo que no
sale literal como si estuviera oculto. Gana (a): imagen fiel (lo que el probador no oculta no está
oculto) y pureza (la regla es la medida, no un deseo). **Reversible** sólo cuando un probador que
oculte lo pruebe con su propio falsador (D-D); nunca porque un valor deje de salir literal.

### D-B — La tabla: lo que sale literal, prueba a prueba

Con la regla de D-A y con control a cero. La cuenta, donde el asiento la registra, es el número de
apariciones en los bytes de la prueba; donde el asiento dice el hecho sin cuenta, la tabla también.

| prueba | lo que sale literal | medido en |
|---|---|---|
| envío (SEND-v1 y SEND-v2) | la clave de gasto (42) y la identidad del receptor; en el v2, además, el saldo, el importe, la sal, el `leaf_salt` y el sobre `X` | §521, §523 |
| cobro (CLAIM) | la clave de gasto (42) y el saldo del receptor antes y después (44 y 44); del pagador, nada (0) | §521, §524 |
| prenda | la clave de gasto (42) | §521 |
| emisión a pendiente | la identidad del receptor y la sal (42 y 42) | §524 |
| autorización delegada de un custodio (emisión, congelación, recuperación) | la clave de SU custodio: tras una sola operación delegada, el nodo tiene dos y puede autorizar la siguiente | §523 |
| gobernanza delegada | la clave de cada miembro (42 y 42; la del que no firma, 0) | §524 |
| umbral conjunto (`circuit_threshold`) | las dos claves (42 y 42; la del custodio que no firma, 0) | §526 |
| auditoría (`prove_minimum` de la capa, con el circuito de `stark-experiment`) | la clave de gasto y el saldo exacto | §523, §524 |
| quema | la clave de gasto | §523 |
| solvencia (`stark-experiment`) | el saldo y el importe (44 y 44) | §524 |
| `double_entry` (`stark-experiment`) | las identidades del emisor y del receptor, los cuatro saldos, el importe y el nonce del emisor (44 cada uno) | §526 |
| banda | el saldo y el `leaf_salt`; la cota superior (`pedido - 1`) es pública por diseño | §521, con su corrección en el §527 |
| edad | el emisor, cuando todos los pendientes son del mismo (43); con emisores distintos, no | §521, §523 |
| sobre de cobro (portable) | la sal, la `X` y el importe exacto | §521 |
| sobre de pago (portable) | la sal, el `delta` y el `refund_id` | §521 |
| camino de merkle | nada: ni la hoja ni los hermanos son constantes | §524 |

El catálogo de rechazos lleva DOS pruebas reales —un SEND y un CLAIM de un banco— de las que se
deriva la clave de gasto de sus dos cuentas sandbox (§521). Son claves de prueba: lo que era falso
era la propiedad publicada, no una filtración hecha.

Gana la tabla frente a una frase general: claridad (un lector dice qué es público prueba a prueba)
y coherencia (cada fila remite al asiento que la midió). **Reversible** fila a fila: una medida
nueva que la contradiga la corrige, en el asiento que la mida y aquí.

### D-C — El principio del API se queda como regla, y hoy no se cumple

La regla 3 del PROCESO —**la clave de gasto no viaja jamás**— no se reescribe. Dos caminos: (a)
mantenerla y declarar dónde se incumple; (b) rebajarla a «no viaja como campo del cable», que la
haría cierta sin que nada cambie. Gana (a): (b) embellece el relato y ensucia el invariante, y la
constitución lo rechaza. La tabla de D-B dice dónde se incumple hoy: en el envío, el cobro, la
prenda, la auditoría y la quema viaja la clave de gasto; en las autorizaciones delegadas, la
gobernanza y el umbral conjunto, las de los custodios. El PROCESO lo dice desde el §521; aquí queda
medido prueba a prueba.

Este RFC no erosiona el principio —lo erosiona el probador— y por eso no nace RETIRADO.
**Reversible** hacia ninguna parte: una regla no caduca. Deja de incumplirse cuando lo pruebe la
suite de E2 contra un probador que oculte.

### D-D — La ocultación, fuera de este RFC

Este RFC dice lo que hay; no elige cómo cambiarlo. Ocultar el testigo pide un probador que
enmascare: el issue 9 de winterfell enumera lo que haría falta —valores aleatorios al final de la
traza, un polinomio aleatorio de grado bajo combinado con el de composición y compromisos de Merkle
con sal—, y eso toca todos los AIR de la casa. Es una primitiva nueva para una propiedad nueva, y va
en un RFC propio. Lo que declaran otros probadores se lee en su portada antes de afirmarlo, y es
trabajo de ese RFC; RISC Zero, además, es un zkVM, que la constitución deja fuera.

Gana dejarlo fuera: pureza (una primitiva por propiedad) y el orden que siguieron los asientos §521
a §526 —primero dejar de afirmar lo falso, después decir qué se promete y sólo entonces cambiar el
probador—. **Reversible** si un probador que oculte resulta caber sin tocar los AIR: entonces cabe
aquí como etapa.

**Revertida en la E3** (sesión 166): el probador que oculta cabe sin tocar ningún AIR —medido en
dos juguetes fuera del árbol, y para los 35 de producción por su censo y las fórmulas leídas; lo
confirman E3a y E3b con sus tests (D-G, D-K)—, así que la ocultación entra aquí como etapa y no
en un RFC propio. La primitiva nueva para la propiedad nueva es el fork, y la declara la D-F: una
primitiva por propiedad, como pedía la pureza.

### D-E — La tabla tendrá su testigo: E2

Una promesa que nadie re-mide caduca en silencio. E2 convierte la tabla de D-B en suite: produce
cada tipo de prueba con claves y valores de alta entropía, cuenta sus apariciones en los bytes y
compara con la tabla, con un control que no puede aparecer y tiene que dar cero. Un circuito nuevo
que ponga un secreto en una columna constante, o un cambio que mueva una fila, la pone roja. Hasta
entonces, la tabla vale lo que valen las lecturas que la midieron.

Gana la suite frente a dejar la tabla como prosa: pureza (testigo antes que promesa) y coherencia
(la misma regla que la suite de cada circuito ya sigue). **Reversible** en su forma —test del
crate o instrumento del canon—, no en su existencia.

### D-F — El probador que oculta es un fork de winterfell 0.13.1

Dos caminos se pesaron (INFORME-PROBADOR-165, corregido en la sesión 166): (b) bifurcar
winterfell 0.13.1 y ocultar dentro; (c) migrar a Plonky3, que trae `HidingFriPcs`. (c) reabre los
35 AIR de producción —10.100 líneas de implementación, 4.341 de transición— y deja fuera a
`EdadAir`, cuyo tramo auxiliar p3-uni-stark no tiene. (b) mide 488 líneas —350 en el fork y 138
propias, la sal y el azar sembrado— y ninguna en un AIR, frente a una referencia de 892 a 1.697
líneas de Plonky3 para lo mismo (SPIKE-B-P4 r2).

Gana (b): pureza (los 35 AIR conservan la semántica que sus suites ya prueban) e imagen fiel (el
coste es el medido, no el esperado). **Reversible** si el código del fork pasa del doble de la
referencia (1.784 líneas), si una auditoría lo tumba o si winterfell publica ocultación propia:
entonces se sigue a winterfell y el fork se retira.

### D-G — La ocultación vive entera en el núcleo: ningún AIR cambia

El issue 9 de winterfell, que su autor abrió en 2021, pide tres piezas; la nota 2024/1037 añade
una cuarta. Las cuatro caben dentro:

- **las filas** (la primera del issue): detrás de las T filas reales, T filas aleatorias. Un
  envoltorio, `Oculta<A>`, presenta al núcleo un AIR de 2T filas con una columna más y las
  exenciones del interno más T, y delega en el interno transiciones, aserciones, tramo auxiliar y
  columnas periódicas. El interno sigue viendo T, así que sus aserciones de última fila no se
  mueven. En el hilo del issue, su autor ya daba esta pieza por cubierta sin tocar apenas el
  diseño de las restricciones;
- **el polinomio aleatorio antes de FRI** (la segunda): la columna de más es aleatoria entera;
  entra en DEEP con su coeficiente, como cualquier columna, y enmascara lo que FRI abre;
- **las hojas** (la tercera): los compromisos de Merkle llevan sal (D-I). El issue la pide para la
  traza; el fork sala también las restricciones y FRI;
- **el cociente** (la nota 2024/1037, apartado 4.2, que el issue no trae): el polinomio de
  composición se parte con paso 2T − m y sus trozos se aleatorizan con polinomios de grado menor
  que m que se cancelan en la suma, con m ≥ 44 —las 42 consultas, z y z·g— y m = 64.

Que ningún AIR cambia lo sostiene el censo de los 35 (PASTE-P4-M y PASTE-GRADOS-M r2): ninguno fija
exenciones propias —todos llevan la de serie, y 1 + T sobre 2T es justo el máximo que winterfell
admite (`air/context.rs`:302-307)—; ninguno usa aserciones periódicas ni de secuencia; ninguno lee
el ancho del marco; ninguno escribe en el meta de la traza; ninguno define un método del AIR que
el envoltorio no delegue; y los 35 probadores usan el evaluador de serie.

Queda la segunda cota de las exenciones (`air/context.rs`:315-327), que depende del grado. Con las
fórmulas de `air/transition/degree.rs`:90-115, un AIR de 2T filas con T + 1 exenciones cabe si
d + Σ(1 − 1/c) ≤ ce + ½. Caben 34 de 35 con su propio ce; `WorkAir`, de grado 3 sin ciclos,
necesita 4 donde winterfell le da 2. El envoltorio sube el ce a la menor potencia de dos que deje
sitio, con tope en el blowup, y lo calcula con las funciones de winterfell. El ce solo lo usa el
probador: medido en un juguete con la forma de `WorkAir`, una prueba hecha con la subida verifica
sin ella, y sin la subida no sale una prueba que verifique (SPIKE-B-P4 r2).

Gana el núcleo frente a ocultar desde fuera, que es lo que hizo la etapa 1 del spike: coherencia
(las suites y los pines de los 35 AIR no cambian) y pureza (una primitiva, un sitio).
**Reversible** AIR a AIR: uno que no quepa —exenciones propias, aserciones periódicas o de
secuencia, o un ce por encima del blowup— lo dice su RFC y se oculta desde fuera.

### D-H — La marca viaja en el meta de la traza, con m dentro

Una prueba oculta lo dice en el meta de su `TraceInfo`, que ningún AIR usa. Con el meta vacío, la
prueba es la de winterfell byte a byte: el fork apagado reproduce sus bytes (medido). Con la
marca, el meta entra en el transcript, y tocar un byte de la marca deja la prueba sin verificar
(medido). El verificador despacha por la marca sin que cambie una sola llamada a `verify`. Con la
marca y una traza que no se puede partir en dos —longitud menor que 16 o ancho menor que 2—
devuelve error sin llegar al envoltorio, porque el kit tiene que fallar cerrado (medido). El m
del cociente viaja dentro de la marca; en los spikes vive en una estática. Antes de usarlos, el
verificador comprueba que conoce la versión de la marca y que 0 ≤ m < 2T, y si no, rechaza. m
decide la ocultación, no la solidez: un m corto solo perjudica a quien prueba.

Gana el meta frente a un campo nuevo de `ProofOptions` o del cable: pureza (las pruebas sin
ocultar no cambian ni un byte) y coherencia (la marca va atada al mismo transcript que todo lo
demás). **Reversible** si algún AIR llega a necesitar el meta: entonces la marca se muda, y lo dice
aquí.

### D-I — La sal se queda hasta que un argumento la retire

La sal cuesta 6.567 bytes por prueba, el 46 % de lo que crece al ocultar; sin ella, la prueba
oculta pesa ×1,12 la de winterfell en vez de ×1,22 (SPIKE-B-P4 r1). El censo literal no puede
juzgarla, porque no ve las hojas. El argumento que la retire lleva dos cuentas separadas: por
columna, 44 puntos abiertos —las 42 consultas, z y z·g— frente a T filas aleatorias; y en DEEP y
FRI, las combinaciones que abren las capas frente a la columna aleatoria que las enmascara.

Gana quedársela: pureza (nada se retira sin su argumento). **Reversible** cuando el argumento esté
escrito y revisado: la sal se va y la prueba adelgaza.

### D-J — Las cifras de bytes pasan a bandas y el azar sale del sistema

Una prueba oculta no es determinista: con otra semilla pesa distinto (E1, E2 y E4 del SPIKE-B-P4:
80.936, 81.637 y 80.807 bytes). Las cifras publicadas de bytes pasan a bandas; los vectores de
`spec/vectors/` necesitan azar sembrado o una vigilancia que no compare bytes; y en producción el
azar sale de la entropía del sistema.

Lo medido sin molienda y frente a winterfell: ×1,22 en bytes con 42 columnas, ×3,2 a ×3,3 al
probar y ×1,9 a ×2,0 al verificar; con 5 columnas, ≈ ×1,29 en bytes (una sola
semilla). Frente a ocultar desde fuera, lo mismo: ×0,98 en bytes y ×1,05 al probar.

Gana la banda frente a una cifra: imagen fiel (una cifra única mentiría). **Reversible** hacia
ninguna parte mientras se oculte.

### D-K — Dos pasos: el fork entra apagado, y encenderlo sube el cable

E3a mete el fork en el árbol apagado: cada prueba sale byte a byte como hoy y el cable no cambia
(medido con el fork apagado en los spikes). Trae, como tests del árbol y con el modo oculto solo
en los tests, los falsadores de los spikes: censo cero, verifica, rechaza público+1, marca tocada,
dimensiones inválidas, `WorkAir` con la subida y la regresión con el fork apagado. Sin ellos, E3a
sellaría código que nada ejercita: testigo negativo antes que la función.

E3b lo enciende para las pruebas que cruzan el cable. Una prueba oculta no la verifica un
winterfell sin bifurcar (razonado; lo ilustra la 2a del spike, donde el verificador sin el paso
del cociente rechaza la prueba, y en los AIR que asertan su ancho en `new` entraría en pánico en
vez de rechazar). Así que la versión sube a `zkssl/0.4` y los vectores de la 0.3 se conservan
bajo su versión (regla 2 del PROCESO).

E3b no se sella sin la suite de E2 contando cero valores literales en cada tipo de prueba. Ese es
el falsador propio que piden la D-A («sólo cuando un probador que oculte lo pruebe con su propio
falsador») y la D-C («cuando lo pruebe la suite de E2 contra un probador que oculte»). Con él, la
tabla de D-B pasa a cero y la D-A se revierte.

Gana partir en dos: coherencia (el orden que siguieron los asientos §521 a §526: primero lo que no
rompe, después lo que sí) y claridad (cada paso con su testigo). **Reversible** en su orden si E2
se retrasa: E3a puede sellarse sola; E3b no.

## Compatibilidad

- `zkssl/0.3` **no sube**. Ningún método, tipo ni error del cable cambia; ningún vector se
  reescribe ni nace.
- La promesa ya había cambiado en la prosa y en los comentarios (§521 a §526): este RFC la fija en
  un sitio.
- **E3a** no sube `zkssl/0.3`: con el fork apagado, las pruebas son byte a byte las de winterfell.
- **E3b** sube a `zkssl/0.4`: las pruebas que cruzan el cable llevan la marca y solo las verifica
  el fork. Los vectores de la 0.3 se conservan; los de la 0.4 nacen con azar sembrado (D-J).

### Por qué entra por RFC

No toca el cable, y el PROCESO no lo exige. Entra porque decide qué afirma el protocolo de sus
propias pruebas, y eso merece su expediente: los asientos §521 a §526 lo dejaron escrito como
deuda.

## Seguridad

- **El principio del API** (regla 3 del PROCESO): hoy no se cumple; D-C dice dónde.
- **El operador es el adversario que recibe las pruebas.** El nodo ve todo lo que dice la tabla de
  D-B, y tras una sola operación delegada tiene las claves de dos custodios, que bastan para
  autorizar la siguiente. La conservación no cae: la prueba sigue siendo sólida. Cae quién puede
  mover el suministro.
- **Quien lea un sobre publicado** ve la sal, la `X` y el importe del cobro, o la sal, el `delta` y
  el `refund_id` del pago. Los vectores de `spec/vectors/` son de sandbox, con claves de prueba.
- **Lo que este RFC nombra y no paga:**
  - el literal del kit «esta causa no publica el saldo: la banda lo prueba sin el»
    (`crates/zk-ssl-verify/src/main.rs`) y la cabecera de `crates/zk-ssl-air/src/pago_en_curso.rs`
    («el plazo NO viaja … nunca `delta`»), que viven en la clausura del kit y esperan al corte que
    la abra;
  - la misma afirmación en la cabecera de `crates/zk-ssl/src/prueba_pago.rs`, que está fuera de esa
    clausura y ningún corte tocó;
  - que `audit`, `prove_minimum` y `burn` reciban la clave de gasto como argumento de un método de
    la capa (`crates/zk-ssl/src/audit.rs`, `crates/zk-ssl/src/burn.rs`);
  - las dos pruebas reales del catálogo de rechazos;
  - la promesa de `doc/ZENODO.md` de que el circuito del tope no revela el saldo, que describe un
    depósito con DOI y no se edita;
  - y la semilla de `zk-core`, que es Groth16 y queda fuera del modelo, como iso-bridge,
    settlement-layer y los experimentos de PLONK y Halo2.
- **Regla 3 del PROCESO:** E3b es la etapa que haría cumplir el principio del API. Hasta que la
  suite de E2 lo mida en cada tipo de prueba, la D-C sigue diciendo dónde se incumple.
- **Lo que el censo no ve:** que no salga nada literal no es ocultación (D-A). Más allá del censo,
  la E3 descansa en el argumento de la D-I y en la construcción de la nota 2024/1037. Ni el fork ni
  la ocultación están auditados (H7).
- **Fallar cerrado:** con la marca, el verificador rechaza con error una traza que no se puede
  partir (D-H). Los AIR que asertan su ancho en `new` —`AuditAir`, `circuit_audit.rs`:366, entre
  otros— entran en pánico ante una prueba con otro ancho, con marca o sin ella. Es anterior a
  esta etapa y queda como punto propio de la cola 5.A.
- **Depuración:** con los asertos de depuración de winterfell encendidos —grados declarados
  iguales a los reales y tamaño del dominio de evaluación, `constraints/evaluation_table.rs`:
  181-230—, los dos juguetes pasan con el envoltorio y la subida, y probar G sin la subida cae en
  el aserto de grados (`:214`): en depuración, olvidar la subida no pasa inadvertido (medido,
  SPIKE-B-P4 r3). En los 35, el tamaño que exigen coincide con el de la subida (razonado con las
  fórmulas leídas). En el árbol los ejercitan los tests del modo oculto de E3a, que corren en
  depuración; la suite con el fork apagado no los toca.

## Referencias

- Los asientos §521 (el testigo se publica: PASTE-521-ROJO, PASTE-ZK-M, PASTE-ZK-M2 y
  PASTE-360-M), §523 (el modelo: PASTE-360-M2 y sus dos falsadores), §524 (PASTE-367-M3) y §526
  (PASTE-367-M4); y el §522 y el §525, los cortes de los comentarios. Los instrumentos viven fuera
  del árbol, en Downloads del autor.
- `spec/rfc/PROCESO.md`, regla 3; `SECURITY.md` §3.bis, donde vive la frase canónica.
- winterfell: la portada del repositorio, <https://github.com/facebook/winterfell>, y su issue 9,
  <https://github.com/facebook/winterfell/issues/9>.
- RFC-0007 (la banda, la edad y el rechazo) y RFC-0008 (los dos sobres portables y la prenda).
- Los instrumentos de la sesión 166, fuera del árbol, con la huella de su salida: PASTE-P4-M
  (`bc2ab57c2a683bcc`), SPIKE-B-ETAPA2A (`bea7d79ec60919f3`), SPIKE-B-P4 r1 (`0afe71ce812aa713`),
  r2 (`5a0d1c5ee86ff89c`) y r3 (`7e4dfda3ced288dc`), y PASTE-GRADOS-M r2 (`112d6654666f2941`); de
  la 165, PASTE-PROBADOR-M, PASTE-FORK-M y SPIKE-B-ETAPA1 (`b51618eaeee76f79`), con el
  INFORME-PROBADOR-165.
- winterfell 0.13.1: `air/context.rs`:290-331 (las exenciones), `air/transition/degree.rs`:90-115
  (los grados) y `constraints/evaluation_table.rs`:181-230 (los asertos de depuración); la nota
  2024/1037, apartado 4.2; el hilo del issue 9, donde su autor da por cubierta la primera pieza.
