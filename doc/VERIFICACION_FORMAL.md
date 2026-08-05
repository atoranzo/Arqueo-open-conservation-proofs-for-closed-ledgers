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

## 5. Estado

FV-0 ejecutada (§183 + SECURITY §2). FV-1: diseñada, sesión propia — el
primer paso es el censo DEL guardián mismo (600+ líneas que parsean; se
extiende lo que se ha leído entero, no lo que se adivina). FV-2: tras
FV-1. FV-3: horizonte.
