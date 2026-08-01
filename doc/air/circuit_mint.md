# Especificación del AIR — `circuit_mint`

**Estado**: segunda escrita. 01-08-2026, contra
`crates/stark-experiment/src/circuit_mint.rs`.

⚠️ **Escrita por el autor del circuito. Hereda sus puntos ciegos y NO es un
contrato.** Ver `AUDITORIA.md` §105.3: una especificación firmada por la
misma mano que el código no contrasta nada; un auditor debe auditar el
código, no este documento.

⚠️ **Si este documento y el código discrepan, el código gana.**

---

## 1. Qué demuestra

> **Dos custodios distintos** del conjunto autorizado, demostrando conocer
> sus claves, **crean `amount` en una cuenta existente** y suben el
> suministro total en la misma cantidad, **sin rebasar el tope**.

Es el enunciado más caro del sistema: **aquí es donde se crea dinero**.

## 2. Disposición

**45 columnas, 512 filas.**

| rango | contenido |
|---|---|
| `0..12`, `12..24` | Estado Rescue, lanes A y B (`LANE_B = 12`) |
| `24`, `25` | `COL_BIT_A/B` — bits del camino **de custodios** |
| `26`, `27` | `COL_KEY_A/B` — claves de los dos custodios |
| `28`, `29` | `COL_IDX_A/B` — sus índices declarados |
| `30`, `31` | `COL_ACC_A/B` — **acumuladores** del índice desde el camino |
| `32..36` | `COL_ACC_ID` — identidad de la cuenta que recibe |
| `36`, `37` | `COL_BAL`, `COL_BAL_NEW` |
| `38`, `39` | `COL_NONCE`, `COL_AMT` |
| `40`, `41` | `COL_SUPPLY_OLD`, `COL_SUPPLY_NEW` |
| `42` | `COL_MAX_SUPPLY` — **el tope** |
| `43`, `44` | `COL_SBIT`, `COL_SACC` — descomposición binaria |

| fila | qué ocurre |
|---|---|
| `0..15` | Hoja de la cuenta: lane A la **vieja**, lane B la **nueva** |
| `15..271` | Ascenso por el árbol de cuentas |
| `271` | `ROW_ACCT_ROOT` — raíces vieja y nueva |
| `272..311` | **Derivación de las dos identidades de custodio** y ascenso por su árbol |
| `311` | `ROW_CUST_ROOT` — raíz del conjunto de custodios |

⚠️ **Los dos lanes tienen papeles distintos en cada fase**: en la primera son
hoja vieja / hoja nueva; en la segunda son **custodio A / custodio B**. La
misma maquinaria sirviendo a dos propósitos, y **eso no está dicho en el
código**.

## 3. Las restricciones que son el enunciado

| ranuras | qué ata |
|---|---|
| `C_INPUT` (10) | Lane A recibe `COL_BAL`; lane B recibe `COL_BAL_NEW` |
| `C_CUST_INPUT` (2) | En `ROW_ACCT_ROOT` entran `COL_KEY_A` y `COL_KEY_B` |
| **`C_ACC`** (2) | El acumulador suma los bits del camino de cada custodio |
| **`C_ACC_FINAL`** (2) | **El índice acumulado = el índice declarado** |
| **`C_BALANCE`** (1) | `BAL_NEW = BAL + AMT` |
| **`C_SUPPLY`** (1) | `SUPPLY_NEW = SUPPLY_OLD + AMT` |
| `C_TRANSPORT` (11) | 11 columnas constantes, **incluida `COL_MAX_SUPPLY`** |
| `C_SEG_LINK` (8) | Descomposición binaria de **ocho** valores |

### 3.1 Los ocho segmentos, que son media especificación

`expected[]` fija qué debe caber en rango:

| # | valor | qué demuestra |
|---|---|---|
| 0-3 | `BAL`, `AMT`, `BAL_NEW`, `SUPPLY_NEW` | caben en el rango del dominio |
| **4** | `MAX_SUPPLY − SUPPLY_NEW` | ⚠️ **el tope no se rebasa** |
| 5-6 | `IDX_A`, `IDX_B` | índices dentro del conjunto |
| **7** | `IDX_B − IDX_A − 1` | ⚠️ **`IDX_B > IDX_A`** |

**Una sola técnica sirviendo a dos propiedades distintas** —el tope monetario
y el orden de los custodios—, y en ninguno de los dos casos hay una
restricción que lo diga directamente.

---

## 4. ⚠️ Qué NO se restringe

### 4.1 Nada compara los índices de custodio directamente — ✅ **comprobado**

**No existe ninguna restricción que diga «`IDX_A ≠ IDX_B`».** La propiedad la
dan **tres cosas separadas**:

1. `C_ACC` acumula los bits del camino de Merkle de cada custodio.
2. `C_ACC_FINAL` exige que ese acumulado **iguale el índice declarado** — el
   camino determina el índice, no se puede mentir.
3. El **segmento 7** exige que `IDX_B − IDX_A − 1` quepa en rango. Con
   índices iguales vale `−1`, que es `p−1` en Goldilocks, **y no cabe**.

✅ **MEDIDO** — `one_custodian_cannot_sign_twice`:

```
Err("verificacion fallo: InconsistentOodConstraintEvaluations")
```

La prueba **se generó, se verificó, y el verificador la rechazó**. La defensa
está en una restricción, **no en el constructor de la traza** — que es la
distinción que §73 costó aprender.

> ⚠️ **Ninguna de las tres da la propiedad sola.** Es la **tercera garantía
> por consecuencia** del proyecto, tras las dos de `circuit_burn`. Y las tres
> comparten fragilidad: **dependen de que las otras piezas no cambien, y nada
> avisa si cambian.**

### 4.2 Nada ata la clave de un custodio a *su* camino — **y no hace falta**

`C_CUST_INPUT` mete `COL_KEY_A` en el lane A y `COL_KEY_B` en el B. Nada dice
que `path_a` sea el camino **de** `key_a`.

✅ **La raíz lo cierra**: la identidad derivada de la clave sube por el camino
dado, y el resultado debe igualar `custodian_set_root`, que es entrada
pública. **Una clave con el camino de otro no llega a la raíz.**

Misma forma que el camino de congelados de `circuit_burn` §4.5.

### 4.3 Nada impide emitir a una cuenta inexistente — **cerrado por `root_old`**

Si la hoja vieja fuera inventada, se crearía una cuenta con saldo de la nada.
Lo impide que esa hoja **suba hasta `root_old`**, que es entrada pública: una
cuenta que no está en el árbol no reconstruye la raíz publicada.

### 4.4 ⚠️ Nada ata `AMT > 0`

Igual que en `circuit_burn` §4.1: una emisión de cero es válida e inútil.
**No crea dinero.**

### 4.5 ⚠️ El tope se comprueba sobre `SUPPLY_NEW`, y eso es correcto **porque
`MAX_SUPPLY` es constante**

El segmento 4 acota `MAX_SUPPLY − SUPPLY_NEW`. Si `COL_MAX_SUPPLY` pudiera
variar entre filas, un testigo lo subiría en la fila donde se comprueba.

✅ **Está en `C_TRANSPORT` (11 columnas) y en las aserciones**, atado a la
entrada pública. **No es falsificable.**

⚠️ Se verificó explícitamente porque **la lista de transporte tiene 11
ranuras y a primera vista solo se contaban 10 columnas**. El undécimo era
justamente `COL_MAX_SUPPLY`. Un descuadre ahí habría sido un fallo de la
clase de §74 —dinero creado fuera del tope—.

### 4.6 Nada impide que los dos custodios sean el **mismo humano**

El circuito demuestra que **dos índices distintos del conjunto** autorizaron.
No puede demostrar que detrás haya dos personas.

**Es una limitación del modelo, no del circuito**, y pertenece a la gobernanza
del conjunto de custodios. Se enuncia porque una especificación que no
distinga «dos claves» de «dos partes» invita a leer más de lo que hay.

---

## 5. Testigos negativos

| test | ataque |
|---|---|
| **`one_custodian_cannot_sign_twice`** | ⚠️ **un custodio firmando dos veces** — nuevo, §107 |
| `no_constraint_is_vacuous` | mutación: toda restricción dispara |
| *(los demás del módulo)* | claves fuera del conjunto, suministro descuadrado, tope rebasado |

### Lo que falta

| ataque | estado |
|---|---|
| Clave de un custodio con el camino de otro (§4.2) | ❌ sin test; **cerrado por análisis** —la raíz— |
| `AMT = 0` | ❌ sin test; inofensivo |
| `MAX_SUPPLY` variando entre filas (§4.5) | ❌ sin test; **cerrado por análisis** — está en el transporte |

---

## 6. Qué obligó a preguntar

- **§4.1**: §80 enunciaba el riesgo del 2-de-N y **nadie lo había probado**.
  Ahora tiene test, y la defensa resultó estar en la descomposición binaria,
  no donde se buscaba.
- **§4.5**: contar las ranuras de `C_TRANSPORT` y ver que no cuadraban con
  las columnas a la vista.
- **§4.6**: la distinción entre «dos claves» y «dos partes», que el enunciado
  del circuito no puede cubrir.

⚠️ **Y dos mediciones malas por el camino** (`AUDITORIA.md` §107.2, §107.3):
un test **vacuo por construcción** —`prove()` no falla en release— que habría
anunciado un fallo inexistente, y una sospecha **acertada por el motivo
equivocado**.
