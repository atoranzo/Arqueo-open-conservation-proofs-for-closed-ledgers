# Verificación formal de los layouts — triaje y diseño

*(AUDITORIA §183; nace de una crítica externa recibida el 2026-08-05)*

## 0. El veredicto, en tres capas

**La crítica es verdadera en su núcleo.** La suite (297+242 en verde)
prueba PUNTOS: testigos válidos verifican; los testigos corruptos que se
nos ocurrieron rebotan. No prueba el universal: que NINGÚN testigo-fantasma
satisface las restricciones para publics falsos. Los circuitos
sub-restringidos son la clase dominante de vulnerabilidad ZK en el mundo
real (el bug de falsificación de Zcash 2018 es el arquetipo). Nuestro §69
(«Qué NO demuestra este documento») exige que este residuo tenga nombre
propio; desde §183 lo tiene, y SECURITY §2 lo lista.

**El mapa de la crítica no era nuestro territorio.** Aquí no hay R1CS ni
selectores Plonkish: es AIR/winterfell, campo Goldilocks, sbox de grado 7.
Las herramientas de estante (Circomspect, Ecne, Picus) viven en el mundo
circom/R1CS; para AIR no existe nada equivalente — si se hace, se hace
casero y en nuestro dialecto, como todo lo demás. `circuit_settlement.rs`
ya no existe (museo, §175-§176). Y el guardián
(`tools/check_constraint_layout.py`) YA cubre la mitad sintáctica de esta
clase: ranuras sanas, periódicas leídas-y-construidas, casos-mutación
sembrados — y nació de bugs reales (las tres ranuras muertas de la
extracción de mint_climb).

**El hueco honesto**: el guardián audita RANURAS; no audita CELDAS. Que
cada celda de la traza (columna × clase-de-fila) tenga dueño es la forma
sintáctica exacta del sub-restringimiento, y es computable a nuestra
escala. Eso es FV-1.

## 1. FV-1 — El censo de celdas (extensión del guardián)

**Propiedad que se verifica** (sintáctica): para cada circuito, para cada
columna `c` de su traza y cada clase de fila `K`, existe al menos uno de:

1. una restricción de transición que **referencia** `next[c]` (o el
   equivalente por offset de carril) bajo un selector activo en `K`;
2. una aserción (`Assertion::single(c, fila, _)`) con `fila ∈ K`;
3. una **declaración de celda libre**: testigo por diseño.

**Clases de fila**, derivadas de las columnas periódicas del propio
circuito (el guardián ya las parsea): filas-hash (`hash_flag=1`), cada
fila-enlace (columnas one-hot: `acct_link`, `link_leaf`, `link_salt`…),
`first_row`, `first_s`/`cont_s`/`seg_link` por segmento, y el resto
(«filas planas», donde solo rigen las restricciones sin selector).

**Declaración de celdas libres** — convención que el guardián parsea,
junto a las constantes del circuito:

```rust
// CELDAS_LIBRES: receptor y salt son testigo (fila 0, cols 4..12) — §178
```

Una celda libre NO declarada = fallo del guardián («celda sin dueño»).
Una declarada que además tenga dueño = aviso (declaración rancia).

**Caso-mutación sembrado** (la tradición del CASO_66): una copia de
`circuit_refund` con una línea `C_CAP` borrada debe hacer gritar al
guardián «celda sin dueño: col 0..4 en clase enlace». Si la mutación pasa
en silencio, el censo está roto.

**Lo que FV-1 NO afirma, en su propia salida**: *referenciada ≠
determinada*. Una celda puede aparecer en una restricción y aun así tener
grados de libertad (cancelaciones algebraicas, selectores multiplicativos
a cero). FV-1 caza la clase empíricamente dominante — la celda que NADIE
mira — no la determinación semántica. Complementa a los discriminantes;
no los sustituye. La salida lo dirá cada vez que corra.

**Compuerta**: se suma a las existentes. Formato:
`28 circuitos · N celdas-clase · 0 sin dueño · K libres declaradas`.

## 2. FV-2 — Spike SMT, acotado y con permiso para fracasar

**Alcance**: UN circuito, el más pequeño (`circuit_refund`: 20
restricciones, 12 aserciones, traza 16×12). Un exportador en `tools/` que
emite el sistema (transiciones con selectores evaluados por fila +
aserciones) en SMT2 sobre `FiniteField` (cvc5 `--ff`, primo Goldilocks).

**Preguntas, en orden de mérito**:
1. **Determinación**: fijados los publics `(P, amount)` y las celdas
   libres de fila 0, ¿está el resto de la traza únicamente determinado?
2. **Consistencia de la cadena**: ¿existe asignación que satisfaga las
   restricciones sin seguir las rondas de Rescue? (Si la respuesta es sí,
   hay fantasma; si el solver no termina, hay acta.)

**Expectativa honesta, declarada de antemano**: grado 7 sobre un primo de
64 bits puede ser intratable para el solver incluso a esta escala. Un
resultado «TIMEOUT con estos parámetros» es un entregable válido y se
registra como tal — el spike compra CONOCIMIENTO del coste, no promete un
verde. Si resulta tratable en el pequeño, la pregunta de escalar a
`circuit_send` (51 columnas) se abre con datos; si no, se cierra con acta.

## 3. FV-3 — El horizonte, nombrado sin prometerlo

Lean4/Coq (formalizar la especificación y demostrar solidez del AIR) o
K-Framework (semántica ejecutable de las transiciones de la capa) son
programas plurianuales que además requieren formalizar Rescue, FRI y la
semántica de winterfell para que el teorema hable del sistema real y no de
un modelo. Se declaran como dirección — el mismo trato que el dinero
cuántico en README — no como deuda. Quien llegue con manos y años, aquí
está el mapa.

## 4. Precisión de especificación (para no verificar el teorema equivocado)

La propiedad «∀ w₁,w₂ válidos ⇒ mismo nullifier» es FALSA como universal:
usuarios distintos producen nullifiers distintos legítimamente. La
propiedad correcta es **determinación funcional**: el nullifier es función
de (clave, dominio) sin grados de libertad extra en el testigo; ídem el
compromiso respecto de (receptor, salt, importe). FV-1 y FV-2 apuntan a
esa forma. Verificar formalmente la especificación equivocada es el otro
modo de fallar de la VF, y se documenta aquí para que no ocurra.

## 6. FV-1: el prototipo, EJECUTADO — y la frontera, MEDIDA

`doc/fv/censo_celdas_prototipo.py` implementa el enfoque sobre
`circuit_refund` y **se ejecutó**. Resultado en dos mitades, las dos
valiosas:

**El enfoque funciona**: resuelve las constantes, parsea las transiciones
(`next[...]`) y las aserciones (`Assertion::single`), y cruza ambas contra
la convención de celdas libres. Sobre `circuit_refund` reporta las 12
columnas cubiertas, sin celdas huérfanas.

**La frontera está MEDIDA, no supuesta**: el caso-mutación —borrar
`C_CAP`— **NO se distingue** en la simplificación de clases del prototipo.
La razón es exactamente la prevista: sin un intérprete que compute «clase
de fila K = las filas donde el selector S vale 1», la clase agregada
«enlace» no aísla las filas donde `C_CAP` era el único dueño; la columna
sigue referenciada por `C_HASH` en la clase «hash», y el censo agregado no
ve el hueco.

**Lo que esto decide para la entrada 71**: FV-1 real NO es un injerto en el
guardián. Es, en su núcleo, **el intérprete de selectores periódicos** —
leer `get_periodic_column_values` fila a fila y derivar, por selector, el
conjunto exacto de filas que activa. Con eso «celda × clase» se vuelve
computable y el mutante se caza. El prototipo convierte la estimación de
§183 en un requisito probado: **el trabajo es el intérprete, no el cruce.**

**Por qué NO está en `tools/` como guardián**: una herramienta que no
distingue su propio mutante no vigila nada, y la cabecera del guardián lo
dice —un barrido que aprueba lo que no entiende es peor que ninguno
(§42.5)—. Vive en `doc/fv/` como evidencia ejecutable de la frontera.

## 7. FV-1: la frontera, CRUZADA — el intérprete caza el mutante

`doc/fv/interprete_selectores.py` implementa el intérprete de selectores
que §6 midió faltar, y **caza el caso-mutación** que el prototipo no
distinguía. Sobre `circuit_refund`: circuito sano, cero celdas huérfanas;
mutante con `C_CAP` borrado, **cuatro celdas sin dueño** —las capacidades
`(0,1,2,3)` en la clase de fila «enlace»— exactamente las que debían
renacer a cero en la fila de enlace y ahora no gobierna nada.

**Cómo cruza la frontera**: deriva las clases de fila del selector REAL, no
de una etiqueta —lee `hash_flag = [1×NUM_ROUNDS, 0]` sobre el ciclo y
obtiene clase «hash» = filas 0..6, clase «enlace» = fila 7— y ubica cada
restricción por su selector (`hash_flag` → hash, `1−hash_flag` → enlace).

**El hallazgo que lo desbloqueó** (§185): una aserción vive en UNA fila,
luego en UNA clase. El prototipo trataba `Assertion::single(c, 0, _)` como
cobertura de la columna `c` en TODA clase, y esa era la fuga: las
capacidades están aseveradas a cero en la fila 0 (clase «hash»), pero
`C_CAP` las gobierna en la fila 7 (clase «enlace») — filas distintas,
clases distintas. Cubrir «toda clase» con una aserción de la fila 0 borra
justo la distinción que caza el sub-restringimiento. Corregido —la
aserción cubre `(columna, clase-de-su-fila)`— el mutante salta.

**Lo que queda de la entrada 71**: generalizar el intérprete a los 28
circuitos —clases de fila de selectores multi-ciclo (`with_cycles`,
one-hot de enlaces de árbol, segmentos), carriles duales (`offset`/
`LANE_B`), y aserciones en filas no-cero— y entonces injertarlo en el
guardián como compuerta. El núcleo difícil —derivar clase-de-fila del
selector y cazar el hueco— está **probado sobre el circuito simple**. El
resto es cobertura, no concepto.

## 8. FV-1: dos carriles — y el falso positivo del alias

`doc/fv/interprete_dos_carriles.py` lleva el intérprete al primer circuito
de dos carriles, `circuit_frozen_climb` (45 restricciones, ascenso de
Merkle). Cuatro piezas nuevas sobre el de un carril: bucle de carril
`for (lane, offset) in [(0,0),(1,LANE_B)]`, índices crudos (`result[24+i]`,
`result[44]`), selector booleano de fila-completa (`current[COL_BIT]*...`,
clase «todas»), y **seguimiento de aliases**. Criterio de §185 cumplido:
circuito sano cero huérfanas, mutante (`result[24+i]` borrado) cazado —
cuatro capacidades del carril A sin dueño en «enlace».

**El hallazgo, el más importante de la serie**: en la primera versión, el
circuito SANO reportó `(24, enlace)` huérfana —`COL_BIT`, el bit de
dirección—. **No era sub-restringimiento: era ceguera del intérprete.**
`COL_BIT` se lee una vez como `let bit = next[COL_BIT];` y ese `bit` entra
en las restricciones de colocación bajo `link_flag`; el parser buscaba
`next[COL_BIT]` DENTRO de cada `result[...]` y no seguía el alias. Un
verificador sintáctico que no sigue variables intermedias **grita
"vulnerabilidad" sobre código sano** —el peor fallo de una herramienta de
seguridad, porque destruye su credibilidad (§137: censar todas las
representaciones; §42.5: no condenar lo que no se entiende)—. Corregido con
cosecha de `let X = next[...]` y propagación; el sano queda limpio y el
mutante sigue saltando.

**Estado de la 71**: el intérprete cubre un carril y dos carriles, con las
cuatro piezas y el seguimiento de aliases. Queda: selectores multi-ciclo
(`with_cycles` con periódicas one-hot de árbol y segmentos de rango — los
circuitos de cuentas), y el injerto final en el guardián como compuerta.
El concepto está probado en las dos topologías; el resto sigue siendo
cobertura, ahora con una trampa —el alias— ya conocida y resuelta.

## 5. Estado

FV-0 ejecutada (§183 + SECURITY §2). FV-1: diseñada, sesión propia — el
primer paso es el censo DEL guardián mismo (600+ líneas que parsean; se
extiende lo que se ha leído entero, no lo que se adivina). FV-2: tras
FV-1. FV-3: horizonte.
