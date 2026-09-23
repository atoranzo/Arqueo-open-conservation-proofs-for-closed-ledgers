# RFC-0009 — Lo que revela una prueba: la promesa mientras el probador no oculte su testigo

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 162, 163, 164, 165, 166, 169, 170 y 171)
- **Fecha:** 2026-09-21
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). Este RFC no
  cambia un método, un tipo del cable ni un vector: cambia lo que se promete de ellos.
- **Asiento(s) de AUDITORIA:** §521 (el testigo se publica), §522 (los comentarios y el literal del
  API), §523 (el modelo de las columnas constantes), §524 (el PASTE-367-M3, y lo que era de
  Groth16), §525 (los comentarios, con el censo que dice de qué sistema habla cada frase) y §526
  (el PASTE-367-M4); el §527, que lo adopta; el §528, que le añade la E3; el §531, que sella
  la E2 con su suite en el árbol y remide la tabla; el §532, que toma la foto del probador prístino
  (D-R); y el §533, que mete el fork en el árbol, apagado (E3a-1).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: qué se promete (D-A), lo que sale literal en cada prueba (D-B), el principio del API como regla que hoy no se cumple (D-C) y la ocultación fuera de este RFC (D-D) | no | sellada en el §527 |
| E2 — el testigo de la tabla | una suite que produce cada tipo de prueba y cuenta sus valores literales contra la tabla de D-B, con un control que tiene que dar cero (D-E; su forma, D-L a D-Q) | no | sellada en el §531 |
| E3a — el probador que oculta, dentro y apagado | el fork de winterfell 0.13.1 en el árbol, con la ocultación entera en el núcleo y sin tocar un AIR (D-F a D-J); apagado, cada prueba sale byte a byte como la de winterfell; y los falsadores de los spikes como tests del árbol, con el modo oculto solo en los tests (D-K); y antes, la foto del probador pristino que el fork apagado tiene que reproducir (D-R) | no | en curso: el corte 0, la foto (D-R), sellado en el §532; el corte 1, el fork apagado, sellado en el §533; el corte 2, los falsadores de D-K, por hacer |
| E3b — encenderlo | las pruebas que cruzan el cable salen ocultas; la tabla de D-B pasa a cero, con la suite de E2 como testigo (D-K) | sí: `zkssl/0.4` | por hacer; espera a E3a |

Las medidas que abrieron este documento son las de los asientos §521, §523, §524 y §526:
lecturas puras que restauraron el árbol con sha y porcelain, con sus instrumentos fuera del
árbol, en Downloads del autor, como los de los RFC anteriores. Desde el §531 la tabla de D-B la
mide el árbol en cada canon: la suite de E2, `crates/zk-ssl/src/instrumento_revela.rs`.

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

Con la regla de D-A y con control a cero. Desde el §531 cada fila la mide, en cada canon, su test de
`crates/zk-ssl/src/instrumento_revela.rs` (E2), con la unidad de D-M: un elemento de columna
constante sale k(q+2) veces y un digest k·q, con q las posiciones únicas que la prueba abre y k
las columnas o bloques que lo llevan. La tabla dice «en dos columnas» o «en dos bloques» donde k
es 2 y «nada» donde todo es 0. Lo público por diseño —lo que el circuito declara en sus entradas
públicas o lo que el sobre, el recibo o la operación llevan en claro— sale igual de literal y va en
su columna (D-Q); lo pequeño —nonces, contadores, nacidos, cotas inferiores— no se cuenta.

| prueba | del testigo, sale literal | público por diseño, y sale igual | no sale | medido en |
|---|---|---|---|---|
| envío (SEND-v1 y SEND-v2) | la clave de gasto, la identidad del emisor, la identidad del receptor, el saldo antes y después, la sal y el `leaf_salt`; en el v2, además, el sobre `X` | el importe, el límite regulatorio y el suministro, este en dos columnas porque el envío no lo mueve | la clave de vista; en el v2, el `refund_id` y el `delta` | §521, §523; §531 |
| cobro (CLAIM-v1 y CLAIM-v2) | la clave de gasto, la identidad del receptor —en dos bloques: la de la cuenta y la del pendiente, que el circuito obliga a ser la misma—, su saldo antes y después, la sal y su `leaf_salt`; en el v2, además, `X` | el importe y el suministro, en dos columnas | la identidad del pagador; en el v2, el `refund_id` y el `delta` | §521, §524; §531 |
| prenda | la clave de gasto, la sal, el importe y `X` | la identidad del prendador | — | §521; §531 |
| emisión a pendiente | la identidad del receptor y la sal | el importe, el suministro antes y después y el máximo de suministro | — | §524; §531 |
| autorización delegada de un custodio (emisión, congelación, recuperación) | la clave de SU custodio: tras una sola operación delegada, el nodo tiene dos y puede autorizar la siguiente | la operación autorizada | la clave del otro custodio y la del que no firma | §523; §531 |
| gobernanza delegada | la clave de cada miembro | la operación autorizada | la del otro miembro y la del que no firma | §524; §531 |
| umbral conjunto (`circuit_threshold`) | las dos claves | — | la del custodio que no firma | §526; §531 |
| auditoría (`prove_minimum` de la capa, con el circuito de `stark-experiment`) | la clave de gasto, el saldo exacto y el `leaf_salt` | la identidad de la cuenta, el umbral y el techo (`2^62 - 1`) | — | §523, §524; §531 |
| quema | la clave de gasto, el saldo antes y después, el `leaf_salt` y la identidad de la cuenta | el importe y el suministro antes y después | — | §523; §531 |
| solvencia (`stark-experiment`) | el saldo y el importe | el límite | — | §524; §531 |
| `double_entry` (`stark-experiment`) | las identidades del emisor y del receptor, los cuatro saldos, el importe y los dos nonces | el límite | — | §526; §531 |
| banda | el saldo y el `leaf_salt` | la identidad de la cuenta y la cota superior (`pedido - 1`); con el pedido en el saldo más uno, saldo y cota valen lo mismo y la cuenta se dobla: es la coincidencia declarada de D-Q | — | §521, con su corrección en el §527; §531 |
| edad | el emisor, cuando todos los pendientes son del mismo | — | con emisores distintos, ninguno; ni el `nacido` ni la hoja de los pendientes | §521, §523; §531 |
| sobre de cobro (portable) | la sal, `X`, el importe exacto y el emisor, que es el índice de su cuenta | la identidad del receptor y el techo de la banda; la cota inferior y el `nacido` también, pero son pequeños | — | §521; §531 |
| sobre de pago (portable) | la sal, el `delta`, el `refund_id` y el emisor, el índice de su cuenta | el importe | `X` | §521; §531 |
| camino de merkle (`stark-experiment`) | nada: ni la hoja ni los hermanos son constantes | — | la hoja y los hermanos | §524; §531 |
| apertura del reembolso y de la des-emisión (v1 y v2) | nada: su traza no lleva columnas constantes | el importe y, en el v2, la apertura (`refund_id`, `delta`) van en el recibo, no en la prueba | la identidad del receptor, la sal y el importe; en el v2, el `refund_id`, el `delta` y `X` | §531 |
| subida de crédito del reembolso | la identidad de la cuenta que recupera el dinero, su saldo antes y después y su `leaf_salt` | el importe | — | §531 |
| subida de la emisión delegada | la identidad de la cuenta, su saldo antes y después y su `leaf_salt` | el importe, el suministro antes y después y el máximo | — | §531 |
| subida de la congelación delegada | nada | — | el índice de la cuenta y la marca de congelada | §531 |
| subida de la recuperación delegada | la identidad vieja, el saldo y el `leaf_salt` | la identidad nueva, que la operación nombra | — | §531 |

El catálogo de rechazos lleva DOS pruebas reales —un SEND y un CLAIM de un banco— de las que se
deriva la clave de gasto de sus dos cuentas sandbox (§521). Son claves de prueba: lo que era falso
era la propiedad publicada, no una filtración hecha.

Gana la tabla frente a una frase general: claridad (un lector dice qué es público prueba a prueba)
y coherencia (cada fila remite al asiento que la midió). **Reversible** fila a fila: una medida
nueva que la contradiga la corrige, en el asiento que la mida, aquí y en la suite, que es la
tabla como datos (D-O). El §531 lo hizo ya con tres celdas que el PASTE-E2-M leyó como «dos
columnas» y eran coincidencias con el suministro (D-Q).

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

**Sellada en el §531**: `crates/zk-ssl/src/instrumento_revela.rs`, veintiún tests de fila y el
censo del cable, en la fila de la capa del canon. Su forma la fijan D-L a D-Q.

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

Cómo entra (§533): por `[patch.crates-io]` desde el `Cargo.toml` raíz, con los nombres y la
versión de crates.io —es lo que un `[patch]` exige, y lo que el gate de H2 mide por nombre—, bajo
`crates/winter-air`, `crates/winter-prover` y `crates/winter-verifier` como miembros del workspace
con su fila en el canon (49, 6 y 0 tests, los de upstream), `publish = false`, la licencia MIT de
winterfell en cada crate y siete ficheros distintos de los publicados más uno nuevo, listados en
sus README. Sustituye a winterfell en todo el workspace, kit incluido: `winter-fri`, `-math`,
`-crypto` y `-utils` siguen siendo los de crates.io.

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

Lo que el árbol lleva desde el §533, y no es todavía esto: el fork entró con las estáticas del
spike —`COCIENTE_M` y `SUBIR_CE` en `winter-air`; `OCULTAR_FILAS`, `SEMILLA_FILAS` y
`SEMILLA_COCIENTE` en `winter-prover`—, apagadas en su valor de nacimiento, y el verificador lee m
de `COCIENTE_M` y despacha por la marca sin versión ni m dentro. Lo mide la foto de D-R en cada
canon. E3a-2 lleva m a la marca, retira las estáticas y pone el encendido en el API del probador;
hasta entonces, una prueba con la marca y m = 0 la verificaría el kit tal cual.

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

### D-L — El universo de la suite es el cable, y lo dice un censo leído de los fuentes

Lo que la suite produce y la tabla tabula son las AIR que el nodo verifica —las trece de la capa,
los `verify::<` de `crates/zk-ssl/src` fuera de los instrumentos, y la del par de umbral, la que
verifica `verify_threshold_pair`— y las cinco del kit (`crates/zk-ssl-air`), que un sobre
publica: diecinueve. Las cuatro experimentales que ya estaban medidas —`ThresholdAir`,
`SolvencyAir`, `DoubleEntryAir` y `MerkleAir`— se quedan, fuera del cable y dichas así. El censo
se lee de los fuentes en cada corrida, no se teclea: una AIR nueva en el cable sin fila, una fila
del cable que ya no verifica nadie o una fila que nombre una AIR que no existe ponen la suite
roja.

Gana el censo leído frente a una lista: pureza (el universo es el ámbito del cambio, y se re-mide)
y coherencia (el mismo censo con el que el PASTE-E2-M abrió la E2). **Reversible** si el cable
deja de poder leerse de los fuentes: entonces la lista se escribe y el censo la cruza.

### D-M — La unidad de la cuenta: k(q+2) y k·q, con q leído de cada prueba

Un elemento de columna constante sale k(q+2) veces —una por posición abierta y dos fuera del
dominio, en z y en z·g— y un digest de cuatro columnas contiguas k·q, porque fuera del dominio la
extensión cuadrática lo intercala. q son las posiciones únicas que la prueba abre
(`num_unique_queries`: de 38 a 42 medidas, y las 42 de `ProofOptions` son el tope) y k las
columnas o bloques que llevan el valor. Medida en el PASTE-E2-M sobre 107 valores sin excepción
(§531). La suite lee q de cada prueba y exige la cuenta exacta: ninguna celda baja a «al menos».

Gana la unidad exacta frente a una cota: pureza (una cuenta que se cumple sin excepción es una
regla) e imagen fiel (un «42 o más» diría menos de lo que se mide). **Reversible** si un probador
cambia la forma de la prueba: entonces la unidad se re-mide y esta decisión la sigue.

### D-N — Un test del crate `zk-ssl` por fila, en release y dentro del canon

La suite vive en `crates/zk-ssl/src/instrumento_revela.rs`, solo de tests, y corre con la fila
de la capa del canon (`cargo test -p zk-ssl --release`): veintiún tests de fila y el censo. Las
pruebas STARK se saltan en depuración (nota 41) y `--release` las corre; el censo corre siempre.
Un rojo junta todas las celdas que se movieron y las nombra.

Gana el test del crate frente a un instrumento aparte del canon: coherencia (la misma puerta que
gatea cada circuito) y claridad (una fila, un test, un nombre). **Reversible** si su coste deja de
caber en la fila (hoy son 6,6 s de tests, §531): entonces pasa a `--largo`.

### D-O — La tabla tiene veintiuna filas, y su fuente es la suite

Las siete AIR que el censo halló sin fila entran así: `RefundAir` y `RefundAirV2` como una fila
—la apertura del reembolso y de la des-emisión, el mismo constructor—, `CreditClimbAir`,
`MintClimbAir`, `FrozenClimbAir` y `RecoveryClimbAir` como una fila cada una —las subidas del
reembolso y de las operaciones delegadas—, y `ClaimAirV2` como variante de la fila del cobro, como
SEND-v2 lo era ya de la del envío. Cada fila dice qué sale del testigo, qué es público por diseño
—y sale igual— y qué no sale; la unidad es la de D-M. La tabla como datos (D-K): cada fila de la
suite es una lista de celdas con su k, y E3b la pone a cero cambiando datos, no código.

Gana reescribirla frente a añadir filas a la vieja: claridad (un lector dice qué es público y qué
testigo, fila a fila) e imagen fiel (la tabla dice lo que la suite mide, con la corrección del
§531). **Reversible** fila a fila, como antes: la corrige el asiento que la mida, aquí y en la
suite.

### D-P — Las dos pruebas del catálogo de rechazos quedan fuera de la suite

El catálogo de rechazos lleva un SEND y un CLAIM de un banco (§521). La suite no los cuenta: son
vectores, no pruebas que la suite produzca, y su fila ya está —envío y cobro—. Siguen dichos en
D-B y en la Seguridad.

Gana dejarlos fuera: pureza (la suite produce lo que cuenta) y coherencia (el ámbito del censo es
el ámbito del cambio). **Reversible** si el catálogo gana pruebas de un tipo sin fila.

### D-Q — Todo escalar grande de una columna constante es una celda, y las celdas valen distinto

Lo que la suite cuenta son valores, no columnas: si dos valores distintos de la traza valieran lo
mismo, la cuenta de uno contaría el otro. Así leyó el PASTE-E2-M «dos columnas» en la quema y en
la emisión a pendiente, donde el suministro —público, y en columna constante— valía lo que el
saldo o el importe de un libro con una sola cuenta; el ENSAYO-531 lo cazó en el envío, con el
suministro igual al saldo del emisor (§531). De ahí la regla: cada fila cuenta también los
escalares públicos que el circuito lleva en columna constante —el suministro, el límite, el techo,
el máximo—, la suite exige antes de contar que ninguna celda que salga valga lo que otra, y cada
montaje elige valores distintos dos a dos. Lo pequeño —nonces, contadores, nacidos, cotas
inferiores— no se puede contar por bytes y no se cuenta; el nonce de la cuenta va en columna
constante en el envío, el cobro, la quema, las subidas con saldo, la auditoría y la banda —leído
en el `build_trace` de cada circuito, no medido— y dice cuántas operaciones lleva la cuenta. La
banda con el pedido en el saldo más uno es la coincidencia declarada que enseña por qué: saldo y
cota valen lo mismo y la cuenta se dobla.

Gana contar lo público y exigir la distinción frente a contar solo el testigo: pureza (una cuenta
que puede contar dos cosas no mide ninguna) y claridad (el lector ve en cada fila qué es público).
**Reversible** si la cuenta pasa a leer columnas en vez de bytes: entonces la distinción sobra.

### D-R — La foto del probador antes que el fork

E3a promete que, apagado, «cada prueba sale byte a byte como la de winterfell». Esa frase necesita
su falsador en el árbol antes de que el fork entre, porque una foto tomada con el fork dentro sería
un verde que se certifica a sí mismo. El corte 0 de E3a (§532) la toma: tres pruebas con entradas
fijas —`WorkAir`, el circuito canónico de winterfell; `BandaAir` y `EdadAir`, dos AIR del cable que
el kit verifica sin el probador—, producidas dos veces sobre `81549de` con winterfell 0.13.1 sin
bifurcar, idénticas entre corridas, verificadas por su juez, y con su tamaño y su blake3 escritos
como constantes en `crates/stark-experiment/src/kat_probador.rs` (PASTE-KAT-M): `work` 3.539 B,
`banda` 49.418 B y `edad` 66.880 B. Los montajes son copias de los positivos de sus tests, no
referencias: la foto no se mueve porque alguien afine un positivo. El probador sin ocultación es
determinista; el que oculte no (D-J).

Gana tomar la foto antes frente a compararse con la referencia del spike: imagen fiel (el falsador
vive en el árbol y corre en cada canon) y pureza (un KAT con entradas fijas, sin azar ni fork).
**Reversible** sólo con winterfell: si la versión clavada cambia, la foto se toma de nuevo con su
asiento, y quien la mueva dice por qué.

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
  el `refund_id` del pago; y en los dos, el índice de la cuenta del emisor, que el sobre calla en
  sus campos (RFC-0008, D-AH) y la prueba lleva (§531). Los vectores de `spec/vectors/` son de
  sandbox, con claves de prueba.
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
- **Regla 3 del PROCESO:** E3b es la etapa que haría cumplir el principio del API. La suite de
  E2 lo mide en cada tipo de prueba desde el §531, y la D-C sigue diciendo dónde se incumple
  hasta que la suite cuente cero.
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
- El §531 y sus instrumentos, fuera del árbol: PASTE-E2-M (`3bb03cdbbb44a56d`, salida
  `af9b1c5d662ab26b`), ENSAYO-531 (`b95a6f856d493203`, salida `2b1888f387367a54`) y
  ENSAYO-531-r2 (`d0b973d5bdc429f8`, salida `ed0ebac80ea101fc`, cargo `841b702722246376`).
- El §532 y su instrumento, fuera del árbol: PASTE-KAT-M (`f7e4b67352204b1a`, salida
  `8faab897d07a7b05`), con los tres `.bin` y sus líneas en `KAT-M-20260922-225843/` de Downloads.
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
