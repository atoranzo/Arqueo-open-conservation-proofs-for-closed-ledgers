## 9. El mapa de lo que falta — tres capas, para la sesión propia de la 71

Escrito el 05-08-2026 tras probar el concepto en las dos topologías simples
(§185/§186). Fija lo que queda con precisión, leído del árbol, para que
quien retome la 71 arranque con plano y no con improvisación. **Corrige una
imprecisión de traspasos previos**: el resto NO es «solo cobertura». La
Capa 1 es un tercer patrón de selector que el intérprete actual no sabe
leer — trabajo de concepto, no de volumen.

### Capa 1 — el intérprete MULTI-CICLO (el trabajo de concepto que queda)

*(EJECUTADA en §188: doc/fv/interprete_multiciclo.py.)*

El intérprete de §185/§186 deriva la clase de fila de un `hash_flag` de
**ciclo corto**: `[1×NUM_ROUNDS, 0]` sobre `CYCLE_LENGTH=8`, que se repite.
Los circuitos de CUENTAS (familia `circuit_mint_climb`, `credit_climb`,
`recovery_climb`, `mint`, `send`, `claim`…) construyen sus periódicas de
otra manera —columnas de **longitud-de-traza** (`vec![zero; TRACE_LENGTH]`,
512 filas) rellenadas por bucles—, y son **cuatro familias de patrón**, no
una:

1. **hash_flag multi-ciclo** (`mint_climb` :líneas de `get_periodic…`):
   `vec![zero; TRACE_LENGTH]`, y `hash_flag[r]=1` sólo si
   `r % CYCLE_LENGTH < NUM_ROUNDS` **y** `r <= ROW_ACCT_ROOT`. La condición
   de ciclo es la misma, pero acotada a la región de hash de la traza (el
   ascenso de cuentas), con cero fuera. El intérprete debe evaluar
   `r % CYCLE_LENGTH` sobre 0..TRACE_LENGTH, no asumir un ciclo que se
   repite hasta el final.

2. **one-hots de enlace de árbol** (`acct_link`, `link_leaf`, `link_salt`):
   columnas con UN 1 en una fila concreta (`ROW_LEAF_LINK`,
   `ROW_SALT_LINK`, `ROW_LEAF_DONE`) o un 1 por nivel del árbol
   (`acct_link[(CYC_ACC + level)*CYCLE_LENGTH + 7] = 1` para cada `level`).
   Cada una define una clase de fila «puntual» o «una-por-nivel». El
   intérprete debe leer esos índices construidos —`ROW_*` resueltos y el
   bucle de `level`— para saber qué filas activa cada selector.

3. **selector de fila-0** (`sel[0]=1`): `first_row`. Clase de una sola
   fila. Trivial una vez que (2) está.

4. **segmentos de rango** (`first_s`, `cont_s`, `seg_link`): para cada uno
   de `NUM_SEGMENTS` segmentos de `SEGMENT_LENGTH=64` filas, `first_s` marca
   la primera fila del segmento, `cont_s` las `SEGMENT_LENGTH-1` primeras, y
   `seg_link` la penúltima (`(seg+1)*SEGMENT_LENGTH - 2`). Definen clases
   «primera-de-segmento», «cuerpo-de-segmento» y «cierre-de-segmento» — la
   maquinaria de Horner de la descomposición binaria. Es el patrón más
   ajeno al modelo de ciclo actual.

**Riesgo declarado**: la arquitectura «clase = patrón de selector de ciclo»
puede necesitar repensarse a «clase = conjunto explícito de filas
computado columna a columna» para absorber (2) y (4). Eso puede ser un
eslabón largo con reversiones —del peso del paso 2 de B13/B14 (§139, tres
`git checkout`)—. **Por eso es sesión propia con censo fresco**, no fin de
jornada.

**Criterio de éxito, invariante desde §185**: sobre `circuit_mint_climb`,
sano cero huérfanas Y un mutante cazado (candidato natural: borrar una
`C_SEG_LINK`, que debe dejar su columna de acumulador sin dueño en la clase
de cierre-de-segmento). Sujeto real: `circuit_mint_climb.rs`, sus
periódicas de 512 filas. El intérprete de dos carriles se lee ENTERO antes
de extenderlo (la trampa del alias, §186, ya está resuelta ahí y no debe
reintroducirse).

### Capa 2 — el injerto en el guardián (convierte demostración en protección)

*(EJECUTADA en §189: el censo es compuerta de tools/check_constraint_layout.py y las MUERTAS son GRAVES.)*

Hoy los tres intérpretes viven en `doc/fv/` como EVIDENCIA; ninguno es
compuerta. Injertar el intérprete generalizado en
`tools/check_constraint_layout.py` —leído entero, sus 751 líneas, antes de
tocarlo— lo hace guardián activo: una línea más en la compuerta del rito
(`28 circuitos · N celdas-clase · 0 sin dueño`), con su caso-mutación
sembrado al estilo del CASO_66/CASO_50 que el guardián ya lleva. Sólo
entonces FV-1 PROTEGE en vez de demostrar. Requiere que la Capa 1 esté
hecha —injertar un intérprete que sólo cubre dos topologías dejaría los 26
circuitos restantes en falso verde, el pecado de §59.2—.

### Capa 3 — FV-2 y FV-3 (intactas, detrás del injerto)

*(FV-2 EJECUTADA en §190, con acta; FV-3 sigue siendo horizonte.)*

**FV-2** (entrada 72): el spike SMT sobre `circuit_refund`, guion completo
en §2 de este doc; «intratable con acta» es entregable válido. Se hace tras
el injerto, no antes: el censo de celdas es la línea barata que corre
siempre; el SMT es la sonda cara y puntual.

**FV-3**: Lean/K, horizonte declarado (§4), el trato del dinero cuántico.

### Resumen de una línea

Falta: el intérprete multi-ciclo (concepto, sesión propia, cuatro familias
de selector desglosadas arriba) → el injerto en el guardián (protección) →
FV-2 (sonda SMT) → FV-3 (horizonte). El concepto está probado en dos
topologías; la tercera es la que aún puede doblar la arquitectura.
