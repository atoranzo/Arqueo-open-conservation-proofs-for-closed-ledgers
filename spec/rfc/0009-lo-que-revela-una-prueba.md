# RFC-0009 — Lo que revela una prueba: la promesa mientras el probador no oculte su testigo

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 162, 163, 164, 165, 166, 169, 170, 171, 172, 173, 174
  176 y 177)
- **Fecha:** 2026-09-21
- **Versión del protocolo afectada:** `zkssl/0.3` → **`zkssl/0.4` en E3b-2 (§538)** (ver
  Compatibilidad). Hasta E3b-2 este RFC no cambió un método, un tipo del cable ni un vector:
  cambió lo que se promete de ellos; E3b-2 cambia la forma de toda prueba que cruza el cable.
- **Asiento(s) de AUDITORIA:** §521 (el testigo se publica), §522 (los comentarios y el literal del
  API), §523 (el modelo de las columnas constantes), §524 (el PASTE-367-M3, y lo que era de
  Groth16), §525 (los comentarios, con el censo que dice de qué sistema habla cada frase) y §526
  (el PASTE-367-M4); el §527, que lo adopta; el §528, que le añade la E3; el §531, que sella
  la E2 con su suite en el árbol y remide la tabla; el §532, que toma la foto del probador prístino
  (D-R); el §533, que mete el fork en el árbol, apagado (E3a-1); y el §534, que cierra E3a: m en
  la marca, las estáticas fuera, el encendido en el API del probador y los siete falsadores de
  D-K como tests (E3a-2, D-S a D-W); el §535, que abre E3b con la sal como tipo, sin encender
  nada (E3b-0, D-X a D-AC); el §536, que da a la foto sus propios probadores y jueces
  (E3b-1, D-Y); el §537, que hace que el probador oculto devuelva `Err`, no pánico, con un
  testigo malo (D-AH); y el §538, que enciende la ocultación en los 23 probadores con fila y
  sube el cable a `zkssl/0.4` (E3b-2, D-AD a D-AG y D-AI a D-AK).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: qué se promete (D-A), lo que sale literal en cada prueba (D-B), el principio del API como regla que hoy no se cumple (D-C) y la ocultación fuera de este RFC (D-D) | no | sellada en el §527 |
| E2 — el testigo de la tabla | una suite que produce cada tipo de prueba y cuenta sus valores literales contra la tabla de D-B, con un control que tiene que dar cero (D-E; su forma, D-L a D-Q) | no | sellada en el §531 |
| E3a — el probador que oculta, dentro y apagado | el fork de winterfell 0.13.1 en el árbol, con la ocultación entera en el núcleo y sin tocar un AIR (D-F a D-J); apagado, cada prueba sale byte a byte como la de winterfell; y los falsadores de los spikes como tests del árbol, con el modo oculto solo en los tests (D-K); y antes, la foto del probador pristino que el fork apagado tiene que reproducir (D-R) | no | sellada: el corte 0, la foto (D-R), en el §532; el corte 1, el fork apagado, en el §533; el corte 2, m en la marca (D-S, D-T), las estáticas fuera y el encendido en `Prover::ocultacion` (D-U) y los siete falsadores de D-K como tests (D-V), en el §534. La sal de D-I no es del fork: es el `VC` del consumidor y va con E3b (D-W) |
| E3b — encenderlo | las pruebas que cruzan el cable salen ocultas, con la sal como `VC` de los probadores y del kit (D-W); la tabla de D-B pasa a cero, con la suite de E2 como testigo (D-K) | sí: `zkssl/0.4` | SELLADA: corte 0 (§535, la sal como tipo, D-X a D-AC), corte 1 (§536, la foto con probadores y jueces propios, D-Y), D-AH (§537, `Err` con testigo malo) y corte 2 (§538: los 23 probadores con fila encendidos con `MerkleConSal` y m 64, D-AD a D-AG; `zkssl/0.4`; los vectores 0.4 desde sus bancos y los 0.3 bajo `spec/vectors/0.3/`; la banda de D-AC; el comparador de D-AI; la tabla de D-B a cero: 22/22 y 100 celdas, D-A revertida) |

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

**REVERTIDA en el §538 (E3b-2), por el camino que esta misma decisión dejó escrito.** Desde el §538
el probador oculta: los 23 probadores con fila en la tabla de D-B producen con `Ocultacion { m: 64
}` y semillas de la entropía del sistema (D-Z, D-AE), y el falsador propio que D-A y D-C exigían —la
suite de E2 contra un probador que oculte— cuenta CERO en las 100 celdas con k > 0 de la tabla, en
las 22 filas que imprimen, en cada canon (PASTE-538-M, R4 y R9a; el árbol desde el §538). Lo que se
promete ahora, con sus palabras: **una prueba de Arqueo que verifica demuestra la transición que
enuncia y no publica literal ningún valor de columna constante de su traza.** Lo que no se promete:
que nada se deduzca de lo que se abre. La construcción (filas aleatorias y cociente cegado, nota
2024/1037, con la cota de D-I: 44 aberturas por columna frente a T ≥ 64 filas aleatorias) es la que
sostiene la ocultación más allá del censo, y ni el fork ni ella están auditados (H7).

**Confianza residual, en una frase:** quien reciba una prueba —y el nodo las recibe todas— ve el
enunciado público, las raíces y lo que el sobre, el recibo o la operación llevan en claro por diseño
(la columna «público por diseño» de la tabla sigue siendo cierta); del testigo no ve ningún literal,
y lo que pueda deducir de las aberturas lo acota D-I, no lo mide E2.

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

**Desde el §538 (E3b-2) la tabla mide cero.** Las 100 celdas con k > 0 de arriba —lo que salía
literal con el probador apagado— cuentan 0 en las 22 filas que imprimen: la suite de E2 pasa 22/22
con su tabla a cero como dato (`instrumento_revela.rs`, S538), y una celda que volviera a contar la
pondría roja. La tabla se conserva tal cual como lo que la 0.3 revelaba, medido asiento a asiento:
un vector es lo que su versión produjo, y la fila de su verdad es la suite, no este texto (D-O).

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

**Deja de incumplirse en el §538 (E3b-2), y en la medida exacta en que la suite lo mide.** La suite
de E2 cuenta cero literales del testigo en las 23 pruebas con fila (100 celdas, 22/22): la clave de
gasto no sale literal en el envío, el cobro, la prenda, la auditoría ni la quema, ni las de los
custodios en las autorizaciones delegadas, la gobernanza y el umbral conjunto. La regla 3 del
PROCESO lo dice así desde el §538, y lo que queda fuera de la medida lo dice D-A: la confianza
residual está en las aberturas y en la construcción, no en el censo.

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
  traza; la sal la pone el compromiso —`MerkleConSal`, el `VC` que probador y verificador reciben
  como tipo, del lado del consumidor y no del fork (medido en la 172: el fork del árbol no lleva
  sal)—, y por eso sala a la vez la traza, las restricciones y FRI; entra en E3b con el cambio de
  `VC` (D-W);
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

Lo que el árbol lleva desde el §533, y no era todavía esto: el fork entró con las estáticas del
spike —`COCIENTE_M` y `SUBIR_CE` en `winter-air`; `OCULTAR_FILAS`, `SEMILLA_FILAS` y
`SEMILLA_COCIENTE` en `winter-prover`—, apagadas en su valor de nacimiento, y el verificador leía m
de `COCIENTE_M` y despachaba por la marca sin versión ni m dentro; con la marca y m = 0, el kit
verificaba una prueba oculta tal cual (medido en la 172: PASTE-E3a2-M, y m no viajaba: la prueba
con m = 64 llevaba los mismos 21 bytes de cabecera que la de m = 0, y verificaba o no según la
estática). Desde el §534 es esto: la marca lleva m (D-S), lo lee un solo sitio (D-T) y las
estáticas no existen (D-U).

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
sellaría código que nada ejercita: testigo negativo antes que la función. Desde el §534 los siete
son tests del árbol, `crates/stark-experiment/src/falsadores_oculta.rs` (D-V).

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

### D-S — La marca lleva m: el prefijo y cuatro bytes, con un solo lector

La marca es `arqueo:oculta:1` —el prefijo, con su versión dentro— seguida de m en cuatro bytes
little-endian: 19 bytes en el meta de la traza. La lee un solo sitio, `winter-air/src/marca.rs`:
`Marca::leer` da nada con el meta vacío (winterfell tal cual), la marca con el prefijo y sus cuatro
bytes, y error con cualquier otro meta —otra versión del prefijo, otro prefijo o un largo que no
es el suyo—. El verificador rechaza ese error antes de construir ningún AIR («meta de traza
desconocido»), y con la marca rechaza, también antes, una traza de largo menor que 16, de ancho
menor que 2 o un m que no cabe en ella (m ≥ 2T), todo con error y sin pánico.

Gana el prefijo con la versión frente a un byte de versión aparte: claridad (la marca se lee en un
volcado) y coherencia (la de E3a-1 ya era `arqueo:oculta:1`). Y gana rechazar todo meta que no sea
la marca frente a caer al AIR de siempre: fail-closed (medido en la 172: con la marca tocada, el
fork apagado caía al AIR de siempre con una traza de ancho 2 y `WorkAir::new` entraba en pánico).
**Reversible** si algún AIR de la casa llega a escribir en el meta (D-H): entonces la marca se
muda y lo dice aquí.

### D-T — Un solo productor de m: el meta de la traza

m lo dice la marca y sólo la marca. El contexto del AIR lo lee del meta de su `TraceInfo` para
partir el cociente en trozos de paso s = L − m (`context.rs`); el dominio del probador lo lee del
mismo meta al construirse y se lo pasa a `segmentar` con la semilla propia del cociente que sólo
el probador oculto pone (`domain.rs`, `composition_poly.rs`); y el verificador lo lee del contexto
del AIR que ya construyó, para el paso de la evaluación del cociente. Ninguna firma que los 35
probadores implementan cambia: `CompositionPoly::new` ya recibía el dominio.

Gana el meta frente a una estática de proceso: pureza (lo que decide la verificación viaja
firmado en el transcript, y una prueba con m = 64 ya no la acepta un verificador «en 0» ni la
rechaza uno «en 64») y coherencia (un productor, tres lectores del mismo byte). **Reversible** con
la D-S.

### D-U — El encendido es del probador: `Prover::ocultacion`, apagado por defecto

El trait `Prover` gana un método provisto, `ocultacion(&self) -> Option<Ocultacion>`, que devuelve
`None`; `Ocultacion` lleva m y las dos semillas —la de las filas y la columna aleatorias y la del
cociente, separadas— y las pone quien llama: en producción, de la entropía del sistema (D-J). Con
`Some`, `generate_proof` desvía a la versión que oculta; con `None`, la prueba es la de winterfell
byte a byte, y ningún probador de ARQUEO devuelve `Some` hasta E3b. Las cinco estáticas del spike
mueren, `SUBIR_CE` incluida: el envoltorio sube el ce siempre que haga falta, y `Oculta::cabe_con`
queda como consulta para el falsador.

Gana el método provisto frente a un campo nuevo de `ProofOptions` o del cable: pureza (las pruebas
sin ocultar no cambian ni un byte, y las opciones siguen diciendo lo que verifica) y coherencia
(los 35 probadores no cambian: heredan el `None`). Lo que se pierde, y se dice: g2 y g3 del
spike —que el ce subido sólo lo usa el probador, y que sin la subida no sale una prueba que
verifique— necesitaban el interruptor; quedan como medidos en el spike (r3, con los asertos de
depuración), y en el árbol `cabe_con(2)` = falso deja dicho que la subida hace falta.
**Reversible** hacia un campo del cable si E3b lo pide.

### D-V — Los siete falsadores, como tests de `stark-experiment`

Los siete de D-K viven en `crates/stark-experiment/src/falsadores_oculta.rs`, traídos de
`spike/src/main.rs` del spike-b-p4r3 (`145e83ecea366126`), donde eran tramos de un binario con
veredicto por rc: el censo cero (80-83), verifica (84), rechaza público+1 (85), la marca tocada
(89: aquí se exige error, y el pánico que el spike daba por bueno es rojo con nombre), las
dimensiones inválidas (97, más m ≥ 2T), WorkAir con la subida (94, 96: ce 2 → 4, cabe, y la prueba
verifica) y la regresión apagada (86, 88 y la estructural: apagado por el API es byte a byte el
probador de la casa; la misma ocultación, los mismos bytes; otra semilla del cociente deja las
raíces de la traza y mueve la de restricciones). Seis corren sobre `WorkAir` con un probador
propio del módulo que devuelve la ocultación que el test le da; el censo cero, sobre un juguete con
una columna constante de la traza y otra del tramo auxiliar, copiado del spike y recortado a lo
que mide (128 filas, 3 + 2 columnas). `winter-air` y `winter-prover` entran como dependencias de
desarrollo del crate, porque `Ocultacion`, `Marca` y `Oculta` no salen por el paraguas `winterfell`,
que no está bifurcado. Corren en depuración y en release. Desde el §537 el módulo lleva dos
falsadores más, los de D-AH.

Gana `stark-experiment` frente a `zk-ssl-air` (pureza: un juez no compila al probador) y frente a
la capa (coherencia: el modo oculto sólo en tests, y el juguete es de circuitos). **Reversible** si
E3b trae el modo oculto a los probadores de la casa: entonces los falsadores se mudan con ellos.

### D-W — La sal es del consumidor y entra en E3b

La sal de D-I no vive en el fork: `MerkleConSal` es un `VectorCommitment` que el probador y el
verificador reciben como tipo (`VC`), del lado de quien los llama —las «138 líneas propias» de
D-F—, y por eso sala a la vez la traza, las restricciones y FRI. Cambia los bytes de toda prueba
que la use, apagada o no, así que no cabe en E3a sin romper la foto: entra en E3b con el cambio
de `VC` de los probadores con fila en D-B (D-Z; aquí decía «los 35») y del kit, en
`zk-ssl-air` (lo que el kit ve), y con `zkssl/0.4`.

Gana decirlo frente a dejar D-G en presente: imagen fiel (el árbol no sala, y lo dice).
**Reversible** con la D-I: si el argumento retira la sal, este punto se va con ella.

### D-X — La sal nace dentro del tipo, de la entropía del sistema

`VectorCommitment::new(items)` llama a `with_options(items, Options::default())` y no admite
semilla; el núcleo del fork construye sus compromisos con `V::new`
(`winter-prover/src/matrix/col_matrix.rs`:285 y `row_matrix.rs`:227) y `winter-fri`, que no se
bifurca (D-F), sus capas también (`prover/mod.rs`:335). Luego la sal sólo puede nacer dentro del
tipo: `MerkleConSal::with_options` toma 32 bytes de `rand_core::OsRng` (`getrandom`) por hoja y los
pasa por `H`. El spike la sacaba de un SplitMix64 sembrado por una estática de proceso
(`spike/src/main.rs`:43-69), la clase que D-U retiró, y declaraba (:49-50) que en producción
saldría de la entropía; upstream no trae un árbol salado (0 apariciones de `salt` en winter-crypto
0.13.1, un solo `impl VectorCommitment`; PASTE-E3b-M, S2).

Consecuencias, dichas: (1) una prueba con sal no se reproduce byte a byte, ni en tests: se vigila
por lo que verifica, lo que rechaza, lo que cuenta la suite de E2 y lo que pesa (D-J ya lo
admitía); (2) un solo tipo para las dos orillas: el kit compila `with_options` y nunca lo llama;
`rand_core 0.6` y `getrandom 0.2` ya estaban en el lock por `winter-crypto → sha3 → digest →
crypto-common`, así que el lock gana una arista y ningún paquete, pero la clausura del kit SÍ los
gana como paquetes —`cargo tree -p zk-ssl-verify -e normal`: 46 → 48, medido por el ENSAYO-535;
un lock no distingue features— y el THIRD-PARTY del artefacto los nombra desde este sello;
`winter-prover` sigue fuera (la puerta de H2, por nombre); (3) el tipo vive en
`crates/zk-ssl-air/src/sal.rs` (D-W), traído del spike (`spike/src/main.rs`:77-209) con siete
testigos que no prueban ningún STARK, y entra SIN que nadie lo declare como `VC` (E3b-0, §535):
ninguna prueba cambia un byte y la foto de D-R sigue verde.

Gana frente a una semilla en `Ocultacion` con un canal por el fork: pureza (nada de estado de
proceso) y coherencia con D-J; el canal no llegaría a FRI sin bifurcarlo. **Reversible** con D-I:
si el argumento retira la sal, el tipo se va con ella.

### D-Y — La foto guarda sus propios probadores prístinos

Los tres probadores de la foto (D-R) —`WorkProver`, `BandaProver` y `EdadProver`— declaran
`MerkleTree<Blake3>`; dos son de producción, y E3b les cambia el `VC` y les enciende la
ocultación: ni con sal determinista ni apagados volverían a dar los bytes de la foto. La foto mide
una sola cosa, «núcleo apagado con `MerkleTree` = winterfell byte a byte», y la sigue midiendo con
probadores PROPIOS de `kat_probador.rs` sobre `BandaAir` y `EdadAir` reales, copiados como ya copia
sus montajes y como hacen los falsadores de D-V con los suyos. `WorkProver` no es de producción:
se queda como está y sigue siendo la referencia de `regresion_apagada`. Es el corte 1 de E3b
(§536). El juez de cada KAT es también propio: el `verify` de winterfell con `MerkleTree`, como
ya hacía `kat_work`, y no el `verificar` del kit, que en el corte 2 pasa a `MerkleConSal` y
rechazaría la foto (D-AB); y el probador de `edad` lleva copiada su traza auxiliar, porque la
función de producción que la construye es privada de `circuit_edad.rs` y un probador propio
no se apoya en ella. El corte 2 no toca `kat_probador.rs`.

Gana frente a retirar `banda` y `edad` de la foto: pureza (el falsador del núcleo no depende de
decisiones de producción) e imagen fiel (la foto dice de qué probador es cada byte).
**Reversible** si la foto deja de ser el falsador del núcleo.

### D-Z — Se encienden los probadores que tienen fila en la tabla de D-B

E3b enciende, con `MerkleConSal` como `VC` y `Some(Ocultacion)` sembrada de la entropía, los
probadores que tienen fila en la tabla de D-B: los del cable y los cuatro experimentales
tabulados, que son los que el censo de D-L (`el_censo_del_cable_da_fila_a_cada_air_verificada`)
nombra más los cuatro de la tabla; es lo que «la tabla pasa a cero» exige. Los que no tienen fila
no cruzan el cable, no ocultan nada a nadie y siguen apagados con `MerkleTree`, y la suite lo
mide: una fila que se encienda cuenta cero, una que no, lo que la tabla dice. Corrige la cuenta de
D-W: no los 35, los de la tabla. Es el corte 2 de E3b, con D-AA a D-AC.

Gana frente a encender los 35: minimalismo (cada pieza con su función) y coste (probar ×2,6 y
verificar ×1,9 en cada fila encendida, medido en el spike: PASTE-E3b-M, S6); frente a encender
sólo los del cable: coherencia con D-K, que promete la tabla entera a cero. **Reversible** hacia
los 35 si un experimento cruza el cable.

### D-AA — `zkssl/0.4` nace en E3b, y el RFC-0006 lo anunció sin consumirlo

La cabecera del RFC-0006 (:9) y su fila de E2 (:225) anunciaron `zkssl/0.4` para la cabeza v4; el
§415 dejó el cable en `zkssl/0.3` (aditivo) y decidió conservar esa cabecera y corregir debajo
(:247). E3b es la primera etapa que sube el cable de verdad: `zkssl/0.4` nace aquí, y este RFC lo
dice en Compatibilidad; la cabecera del 0006 no se toca (pasado, y ya corregido en su sitio).

Gana decirlo: claridad (dos RFC no pueden reclamar la misma versión sin que uno diga que el otro
no la consumió). **Reversible** hacia ninguna parte.

### D-AB — Los vectores de la 0.3 se conservan bajo su versión, y el kit 0.4 no los verifica

Con `MerkleConSal` en las cinco `verificar` de `zk-ssl-air`, las aperturas de una prueba 0.3 ya no
se leen, y una prueba sin marca en el cable 0.4 se rechaza: fail-closed, sin doble despacho por la
marca. Los 34 vectores con prueba (edad 10, pago 8, pendiente 8 y rechazo 8, medidos) se conservan
bajo su versión (regla 2 del PROCESO) y los verifica el kit que los vio nacer,
`arqueo-verify-v0.2.0` (§442), medido desde fuera; los de la 0.4 se regeneran desde sus bancos con
`--guardar` (§467), nace `zkssl-0.4.json`, y el canon exige «0.4 IDÉNTICO / 0.3 RECHAZADO».

Gana frente a aceptar las dos aperturas según la marca: pureza (un cable, una forma) y fail-closed;
frente a reescribir los vectores viejos, imagen fiel (un vector es lo que su versión produjo).
**Reversible** si una segunda implementación exige verificar 0.3 con el kit vivo.

### D-AC — Las cifras publicadas de bytes pasan a banda, con sus dos atados

`PUBLICADA_PAGO_B` (`crates/zk-ssl/src/metrics.rs`:75, 133.431) y sus dos atados —el test
`la_cifra_publicada_sigue_siendo_la_medida` y `tools/check_publicadas.py`— se miden hoy sobre
pruebas deterministas. Con la ocultación una prueba pesa según `q` y según la sal (D-J: 80.936,
81.637 y 80.807 bytes en el spike): la constante pasa a una banda medida sobre N pruebas, el test la
ata al instrumento y `check_publicadas` la ata a los documentos, que la citan como banda; las diez
citas vivas (medidas) se reescriben en el corte 2, con los `~62 KB` de `metrics.rs`:513 y :517.

Gana la banda frente a una cifra: imagen fiel (D-J). **Reversible** hacia ninguna parte mientras
se oculte.

**Medido en el §538.** Quince muestras por eje (cinco por la vía de la capa y diez por la del
cliente, PASTE-538-M R12): envío 77.444..80.232 B, cobro 76.192..79.736 B; con un margen declarado
del 5 % por cada lado, la banda que el árbol ata es envío 73.571..84.244, cobro 72.382..83.723 y
pago 145.953..167.967 B (139,2 a 160,2 MiB por mil pagos; 146,0 a 168,0 MB en SI).
`PUBLICADA_PAGO_B` se parte en `PUBLICADA_PAGO_MIN_B` y `PUBLICADA_PAGO_MAX_B`; el latido del nodo y
su atado consumen el máximo (adelanta el aviso de la nota 22, que es su sentido);
`check_publicadas.py` ata cada cita viva a uno de los dos extremos y saca `BACKLOG.md` de su
universo como registro, como ya lo estaba del ATADO C. La cifra apagada (133.431 B, 127,2 MiB) queda
en los asientos y en la entrada 22.

### D-AD — La forma oculta es la única forma del cable 0.4

Una prueba oculta declara ancho + 1 (la columna de ocultación) y 2T filas: es lo que `Oculta::new`
construye y lo que la guarda de forma del S529 —`comprobar_forma` en la capa, las cinco
comparaciones del kit y `forma_ok` del par de umbral— rechazaba con la forma apagada del enunciado
(5.A-393, medido en el PASTE-E3b2-M: la capa rechazaba el envío v2 encendido con `MerkleTree` y con
`MerkleConSal`, y lo aceptaba en cuanto la guarda exigía la oculta). Desde el §538 la forma exigida
en `zkssl/0.4` es la OCULTA, derivada en la propia función de la del AIR (`ancho + 1`, `2 *
longitud`; el tramo auxiliar y sus aleatorios no cambian), y la apagada se rechaza: dos falsadores
lo dicen —`guarda_forma` con la forma oculta (45, 1024) como la exacta y (44, 512) rechazada, y el
par mutado a la forma apagada da `WrongTraceWidth`—.

Gana una sola forma frente a admitir las dos por la marca: pureza (un cable, una forma; menos eras
silenciosas) y fail-closed (coherente con D-AB). **Reversible** hacia el doble despacho por la marca
si una segunda implementación no pudiera ocultar.

### D-AE — `Ocultacion` se construye en `stark-experiment`, con `winter-prover` normal

Los 23 probadores con fila devuelven `Some(crate::ocultacion_encendida())`: m = 64 y dos semillas de
`zk_ssl_air::sal::semilla()`, la misma entropía que la sal (D-X). `winter-prover` pasa de dev-dep a
dependencia normal de `stark-experiment` (ya estaba en su clausura por `winterfell`: el lock no se
mueve, `f8c226859ec029af`), y `Ocultacion` no sale por el paraguas `winterfell`.

Gana un solo constructor frente a uno por circuito: coherencia (el mismo patrón en los 23, y en la
fila de D-B se lee de dónde sale cada uno). **Reversible** hacia un m por AIR si D-AG no bastara.

### D-AF — El tipo llega a la capa por `stark-experiment`, sin dependencia nueva

`pub use zk_ssl_air::sal::MerkleConSal` en `stark-experiment`; la capa lo importa de ahí (no depende
de `zk-ssl-air`: llega a los AIR por `stark-experiment`, medido). Un solo tipo para las dos orillas
(D-X): los 21 `verify` de producción —15 de la capa, el del par y los 5 del kit—, los dos de
`metrics.rs` y los 69 de los tests de los 23 circuitos verifican con `MerkleConSal<Blake3>`; los 16
probadores apagados y los dos de la foto siguen con `MerkleTree` (18, censo del generador).

Gana el reexport frente a una dependencia nueva: minimalismo. **Reversible** hacia ninguna parte
mientras la capa no dependa de `zk-ssl-air`.

### D-AG — `m` no cabe en las trazas cortas: acolchado a 64 y `edad` con m ≥ 3

El fork exige m < 2T y desde el §537 lo devuelve como `Err(OcultacionNoCabe)`; con m = 64 caían
`RefundAir` (T = 2 × 8 = 16), `RefundAirV2` (4 × 8 = 32) y `EdadAir` cuando `filas = 8 << m` queda
por debajo de 64 (5.A-395; el banco monta libros con `nextPending = 2`: m = 1). La T de los 23,
leída de sus constantes: 1024 (6), 512 (9), 256 (3), 64 (2), 32, 16 y 8 × hojas; y D-I pide además T
≥ 44. Desde el §538 los dos reembolsos llevan `CICLOS = 8` (64 filas: los merges y el acolchado
detrás, que nada aserta; `ROW_P` fijo en el último merge) y `edad` cuenta al menos 8 hojas en
`celdas_del_libro` Y el kit sube el suelo de `m_canonico` y de `comprobar_enunciado` de 1 a 3,
porque deriva m de `nextPending` (D-2 de E4b-2, «una marca, un subárbol»): subir el probador sin el
kit rechazaba la marca y subir el kit sin el 0.4 rechazaba los positivos de `edad` 0.3 (8/11
medido). Por eso D-AG no es separable del 0.4 y va con E3b-2 (PRECISION 717). Medido: 12/12 en los
reembolsos, 7.401 → 21.014 B y 10.147 → 24.732 B ocultos; `banco_edad` VERDE con m = 3 (79.060 y
75.048 B frente a 43.092 y 44.935 de los vectores 0.3).

Gana el acolchado a 64 con m = 64 frente a un m propio con suelo 44: pureza (una sola regla, sin
excepción por circuito) y claridad. **Reversible** hacia un m por AIR (D-AE lo admite).

### D-AH — Un testigo malo es un `Err` del probador, nunca un pánico

Con la ocultación encendida, una traza que no cumple sus restricciones hacía saltar dos asertos
del fork: el de `m < 2T` en `generate_proof_oculto` y el del cociente oculto en
`composition_poly.rs` («el polinomio no cabe en N trozos de paso 2T − m»). Winterfell apagado
emitía en release una prueba inválida que el verificador rechazaba con error; el fork encendido
se caía (5.A-396, medido en el M3 de la 175 sobre los tres
`t487_una_congelada_no_*_con_camino_ajeno`). Un cliente con testigo malo no debe caerse.

Desde el §537 el probador oculto comprueba la traza REAL contra el AIR interno del envoltorio
(`Oculta::interno`) antes de ocultarla —las aserciones y las transiciones, tramo auxiliar
incluido, con `Trace::comprobar`, la gemela de `validate` que devuelve `Err` en vez de asertar— y
lo hace siempre, también en release: cuesta una evaluación de cada restricción por fila, sin
FFT. Un testigo malo es `ProverError::UnsatisfiedTransitionConstraintError(paso)`,
`AsercionNoSatisfecha` o `RestriccionAuxNoSatisfecha`; una traza corta para ese m es
`OcultacionNoCabe { m, filas }`; un envoltorio cuya segunda cota de las exenciones no cabe ni con
el blowup, `EnvoltorioNoCabe`. Los asertos del cociente quedan como invariantes que una traza
comprobada no puede violar. Apagado, nada cambia: el probador no comprueba y las pruebas siguen
byte a byte (la foto de D-R).

Dos falsadores más en `falsadores_oculta.rs` (nueve con los siete de D-V): una celda falseada en la
fila 100 de `WorkAir` devuelve `Err` en el paso 99, no pánico; 16 filas con m = 64 devuelven
`OcultacionNoCabe { 64, 32 }`, el caso de los reembolsos antes de D-AG. Medido en el PASTE-537-M
(sesión 176): con el encendido entero del corte 2, los tres `t487` reciben
`prove: UnsatisfiedTransitionConstraintError(544)` del probador en vez de caerse, y desde el corte 2
esperan ese `Err` donde hoy esperan una prueba.

Gana `Err` frente a asertar: fail-closed sin pánico rige también dentro del fork (PRECISION 714).
**Reversible** hacia devolver el error desde el propio `segmentar`, que obliga a enhebrar `Result`
por `CompositionPoly::new`, `DefaultConstraintCommitment::new` y los `impl Prover` de la casa: más
ancho, la misma promesa. D-AD a D-AG —la forma oculta como única forma de la 0.4, el constructor
de `Ocultacion`, el reexport por `stark-experiment` y las trazas cortas— llegan con el corte 2.

### D-AI — El comparador 0.4 cruza lo que el circuito fija y declara lo que no se reproduce

El escenario de conformidad era determinista de punta a punta, prueba incluida, y su cadena lo
ataba: `proof_digest = digest_of_proof(proof)` sobre los bytes de la prueba (`log.rs`), `chain` lo
arrastra y `epoch_digest` lo lleva dentro. Con la ocultación encendida los bytes de una prueba no se
reproducen (D-X). Medido en el §538 (PASTE-538-M R11): dos `--emit` del mismo escenario dan iguales
`spec`, `sellado`, `escenario`, `canon`, `supply`, `pending` y, en las seis entradas, `seq`, `kind`,
`root_old`, `root_new` y `compromiso`; difieren exactamente en `proof_digest` y `chain` de las dos
entradas con prueba (seq 0x4 y 0x5) y en `epoch_digest`; el comparador de la 0.3 daba «conformidad
ROTA: 3 diferencia(s)». Desde el §538 `--check` cruza lo que el circuito fija —`seq`, `kind`, las
dos raíces, el compromiso, el suministro y el pendiente— y DECLARA no reproducibles los tres
digests; el vector 0.4 los lleva como lo que su emisión produjo (D-AB), y el canon exige «0.4 → todo
IDENTICO (en lo que el circuito fija)» y «0.3 RECHAZADO (otra version)».

Gana declarar frente a cambiar lo que la cadena ata: pureza (lo que el circuito no restringe no es
un invariante: los bytes de una prueba no lo son) y minimalismo (la cadena del registro no cambia de
formato). **Reversible** hacia una cadena que ate el enunciado en vez de los bytes de la prueba, que
es un cambio de formato del registro y de la cabeza, con su RFC.

### D-AJ — El lado caro deja de afirmarse: encendidos, los dos lados cuestan lo mismo

`el_lado_caro_es_el_declarado` afirmaba el SENTIDO (el receptor paga más, medido apareado el
2026-09-19 con 260-290 ms de envío y 160-190 de cobro). Encendida la ocultación, cada lado cuesta
700-750 ms y el sentido cayó dentro del ruido: una de tres corridas dio PAGADOR (envío 741,6 ms y
cobro 696,9 como mínimos, razón 1,064) y las otras 0,975 y verde (PASTE-538-M R4 y R12). Un contrato
que cae una de cada tres veces no afirma nada: desde el §538 el aserto se retira y se cita (molde
S247), `LADO_CARO` se retira con su historia, los tiempos se imprimen y el test pasa a llamarse por
lo que ata, `los_dos_lados_del_pago_atan_la_banda`: los bytes de los dos lados por la vía del
cliente, dentro de la banda de D-AC (una fila del `--list` de la capa cambia de nombre, ninguna nace
ni muere).

Gana retirar frente a una banda temporal: imagen fiel (los tiempos dependen de la máquina y el
sentido ya no se sostiene). **Reversible** si el sentido volviera a medirse estable.

### D-AK — Los vectores 0.4 los produce el bloque, y las anclas nacen después

Con pruebas no deterministas un vector no se embebe por huella predicha: el BLOQUE-538 corre los
cuatro bancos (`banco_pago`, `banco_pendiente`, `banco_rechazo`, `banco_edad`, con `--guardar`),
renombra las capturas de rechazo a los nombres del catálogo con el mapa derivado del propio banco
(el texto de cada `niega` contra la línea del MANIFIESTO), pasa el arnés línea a línea sobre lo
fresco con el kit 0.4 y solo con 9/9, 9/9, 84/84 y 11/11 comitea; los catálogos 0.3 de pago,
pendiente y edad se apartan enteros bajo `spec/vectors/0.3/<familia>/` (R10: todos sus ficheros
cambian, la cabeza es otra) y de rechazo los nueve que el banco regenera, con un MANIFIESTO de nueve
líneas; los 75 restantes de rechazo se quedan (82/84 con el kit 0.4: sus textos no dependen de la
forma). Los MANIFIESTOS 0.3 sirven a los vectores 0.4 línea a línea (medido: 9/9, 9/9, 11/11 en la
SALIDA-538M), así que el 0.4 es el 0.3 con su cabecera. Todo lo demás del commit se predice; las
anclas de los vectores se miden en el asiento después.

Gana producirlos en el bloque frente a llevarlos en el bloque: fail-closed (sin el arnés en verde no
hay commit) e imagen fiel (un vector es lo que su versión produjo). **Reversible** hacia vectores
sembrados si D-X cambiara.

## Compatibilidad

- `zkssl/0.3` **no subió** hasta E3b-2; **desde el §538 el cable es `zkssl/0.4`**: ningún
  método, tipo ni error del cable cambia, pero cambia la FORMA de toda prueba que viaja (ancho + 1
  y 2T, D-AD) y con qué se verifica (`MerkleConSal`, D-AF); ningún vector se reescribe: los de la
  0.3 se conservan bajo `spec/vectors/0.3/` y los de la 0.4 nacen de sus bancos (D-AB, D-AK).
- La promesa ya había cambiado en la prosa y en los comentarios (§521 a §526): este RFC la fija en
  un sitio.
- **E3a** no sube `zkssl/0.3`: con el fork apagado, las pruebas son byte a byte las de winterfell,
  y apagado es lo que `Prover::ocultacion` devuelve en los 35 probadores de la casa (D-U).
- **E3b** sube a `zkssl/0.4`: las pruebas que cruzan el cable llevan la marca y solo las verifica
  el fork. Los vectores de la 0.3 se conservan bajo su versión y los verifica el kit que los vio
  nacer; los de la 0.4 nacen como capturas de sus bancos, con azar de la entropía, y se vigilan
  sin comparar bytes (D-X, D-AB).
- **E3b-0** (§535) no sube nada: `MerkleConSal` entra como tipo y nadie lo declara como `VC`;
  ninguna prueba cambia un byte. `zkssl/0.4` es de E3b aunque el RFC-0006 lo anunciara para su
  E2 y no lo consumiera (D-AA).
- **E3b-1** (§536) no sube nada: la foto de D-R gana sus propios probadores y jueces con
  `MerkleTree`; ninguna prueba cambia un byte y la foto sigue 3/3 (D-Y).
- **D-AH** (§537) no sube nada: apagado, el probador no comprueba y ninguna prueba cambia un
  byte; encendido —hoy sólo en los falsadores—, un testigo malo devuelve `Err` en vez de
  entrar en pánico.
- **E3b-2** (538) sube a `zkssl/0.4`: los 23 probadores con fila encendidos (D-Z, D-AE), la forma
  oculta como única forma (D-AD), `MerkleConSal` en las dos orillas (D-AF), los reembolsos a 64
  filas y `edad` con m ≥ 3 en el probador y en el kit (D-AG); el `zkssl_protocolVersion`, el OpenRPC
  y el escenario de conformidad dicen `zkssl/0.4` con su comparador (D-AI); los vectores 0.3 se
  conservan bajo `spec/vectors/0.3/` y los 0.4 nacen de sus bancos (D-AB, D-AK); las cifras de bytes
  son una banda (D-AC) y el lado caro deja de afirmarse (D-AJ); la tabla de D-B mide cero (D-A,
  D-C).

### Por qué entra por RFC

No toca el cable, y el PROCESO no lo exige. Entra porque decide qué afirma el protocolo de sus
propias pruebas, y eso merece su expediente: los asientos §521 a §526 lo dejaron escrito como
deuda.

## Seguridad

- **El principio del API** (regla 3 del PROCESO): desde el §538 se cumple en lo que la suite de E2
  mide —ningún valor de columna constante sale literal en ninguna de las 23 pruebas con fila, 100
  celdas a cero— y no más: D-C dice qué se mide y D-A qué confianza residual queda.
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
- **Regla 3 del PROCESO:** E3b-2 (§538) es la etapa que lo hizo cumplir en lo medido: la suite
  de E2, que lo mide en cada tipo de prueba desde el §531, cuenta cero desde el §538 (D-C).
- **Lo que el censo no ve:** que no salga nada literal no es, por sí solo, ocultación (D-A, hasta
  el §538; desde entonces la sostiene la construcción encendida). Más allá del censo,
  la E3 descansa en el argumento de la D-I y en la construcción de la nota 2024/1037. Ni el fork ni
  la ocultación están auditados (H7).
- **Fallar cerrado:** con la marca, el verificador rechaza con error una traza que no se puede
  partir o un m que no cabe en ella (D-H), y desde el §534 rechaza, antes de construir ningún AIR,
  un meta que no está vacío ni es la marca (D-S). Los AIR que asertan su ancho en `new` —`AuditAir`,
  `circuit_audit.rs`:366, entre otros— entran en pánico ante una prueba sin marca y con otro ancho.
  Es anterior a esta etapa y queda como punto propio de la cola 5.A.
- **Depuración:** con los asertos de depuración de winterfell encendidos —grados declarados
  iguales a los reales y tamaño del dominio de evaluación, `constraints/evaluation_table.rs`:
  181-230—, los dos juguetes pasan con el envoltorio y la subida, y probar G sin la subida cae en
  el aserto de grados (`:214`): en depuración, olvidar la subida no pasa inadvertido (medido,
  SPIKE-B-P4 r3). En los 35, el tamaño que exigen coincide con el de la subida (razonado con las
  fórmulas leídas). En el árbol los ejercitan los tests del modo oculto de E3a
  (`falsadores_oculta.rs`, §534), que corren en depuración y en release; la suite con el fork
  apagado no los toca.

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
- El §534 y sus instrumentos, fuera del árbol: PASTE-E3a2-M (`b97ceff1632aa603`, salida
  `b190fdc212bf5271`, cargo `be5669c380e1df63`) con su `meta_m.rs` (`809004b72c1f0587`), que
  midió el meta byte a byte y que m no viajaba; y `spike/src/main.rs` del spike-b-p4r3
  (`145e83ecea366126`), del que salen los siete de D-V.
- El §535 y su instrumento, fuera del árbol: PASTE-E3b-M (`1f1bb5ba5030c04b`, salida
  `ae34451f8cca3a12`, cargo `d934ab3e51f46e2f`, la región de la sal `34508a89201ca6d3`), que leyó
  la sal en el spike y el trait en el registry, y corrió el spike (VEREDICTO VERDE, la sal 6.567
  bytes por prueba).
- El §538 y sus instrumentos, fuera del árbol: PASTE-538-M (`e94784996beeddc7`, salida
  `d92b91297a878cc1`, cargo `93d716f89a286f52`), que midió el corte entero sobre una copia de
  `0eda58c` —las siete suites con cero rojos, los cuatro bancos, las capturas contra el catálogo, el
  escenario dos veces y la banda—, y sus cuatro generadores `gen_m3.py`, `gen537.py`, `gen538.py` y
  `gen538b.py`, que el editor del corte lleva como cargas.
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
