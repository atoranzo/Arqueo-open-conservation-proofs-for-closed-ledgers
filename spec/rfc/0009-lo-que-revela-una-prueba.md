# RFC-0009 — Lo que revela una prueba: la promesa mientras el probador no oculte su testigo

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 162, 163, 164 y 165)
- **Fecha:** 2026-09-21
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). Este RFC no
  cambia un método, un tipo del cable ni un vector: cambia lo que se promete de ellos.
- **Asiento(s) de AUDITORIA:** §521 (el testigo se publica), §522 (los comentarios y el literal del
  API), §523 (el modelo de las columnas constantes), §524 (el PASTE-367-M3, y lo que era de
  Groth16), §525 (los comentarios, con el censo que dice de qué sistema habla cada frase) y §526
  (el PASTE-367-M4); el §527, que lo adopta.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: qué se promete (D-A), lo que sale literal en cada prueba (D-B), el principio del API como regla que hoy no se cumple (D-C) y la ocultación fuera de este RFC (D-D) | no | sellada en el §527 |
| E2 — el testigo de la tabla | una suite que produce cada tipo de prueba y cuenta sus valores literales contra la tabla de D-B, con un control que tiene que dar cero (D-E) | no | por hacer |

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

### D-E — La tabla tendrá su testigo: E2

Una promesa que nadie re-mide caduca en silencio. E2 convierte la tabla de D-B en suite: produce
cada tipo de prueba con claves y valores de alta entropía, cuenta sus apariciones en los bytes y
compara con la tabla, con un control que no puede aparecer y tiene que dar cero. Un circuito nuevo
que ponga un secreto en una columna constante, o un cambio que mueva una fila, la pone roja. Hasta
entonces, la tabla vale lo que valen las lecturas que la midieron.

Gana la suite frente a dejar la tabla como prosa: pureza (testigo antes que promesa) y coherencia
(la misma regla que la suite de cada circuito ya sigue). **Reversible** en su forma —test del
crate o instrumento del canon—, no en su existencia.

## Compatibilidad

- `zkssl/0.3` **no sube**. Ningún método, tipo ni error del cable cambia; ningún vector se
  reescribe ni nace.
- La promesa ya había cambiado en la prosa y en los comentarios (§521 a §526): este RFC la fija en
  un sitio.

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

## Referencias

- Los asientos §521 (el testigo se publica: PASTE-521-ROJO, PASTE-ZK-M, PASTE-ZK-M2 y
  PASTE-360-M), §523 (el modelo: PASTE-360-M2 y sus dos falsadores), §524 (PASTE-367-M3) y §526
  (PASTE-367-M4); y el §522 y el §525, los cortes de los comentarios. Los instrumentos viven fuera
  del árbol, en Downloads del autor.
- `spec/rfc/PROCESO.md`, regla 3; `SECURITY.md` §3.bis, donde vive la frase canónica.
- winterfell: la portada del repositorio, <https://github.com/facebook/winterfell>, y su issue 9,
  <https://github.com/facebook/winterfell/issues/9>.
- RFC-0007 (la banda, la edad y el rechazo) y RFC-0008 (los dos sobres portables y la prenda).
