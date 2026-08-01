# Especificación del AIR — `circuit_burn`

**Estado**: primera de veintisiete. Escrita el 01-08-2026 contra el código en
`crates/stark-experiment/src/circuit_burn.rs`.

> Es el contrato contra el que auditar este circuito. `SECURITY.md` §3.1 lo
> llama la carencia de prioridad más alta del proyecto: **un fallo de solidez
> es dinero falso invisible**, y hasta este documento no había nada contra lo
> que contrastar el código salvo el propio código.

⚠️ **Si este documento y el código discrepan, el código gana y este documento
está mal.** Se escribe leyendo el código, no al revés.

---

## 1. Qué demuestra

> El titular de una cuenta, **demostrando conocer su clave de gasto**,
> retira `amount` de su saldo y **reduce el suministro total en la misma
> cantidad**, sin estar congelado.

La prueba convence al verificador de que existe un testigo —clave, saldo,
nonce, caminos de Merkle— consistente con las raíces públicas antes y
después.

## 2. Disposición de la traza

**Ancho: 42 columnas. Longitud: 512 filas.**

### 2.1 Columnas

| rango | contenido |
|---|---|
| `0..12` | **Estado Rescue, lane A** — la hoja **vieja** y su camino |
| `12..24` | **Estado Rescue, lane B** (`LANE_B = 12`) — la hoja **nueva** |
| `24` | `COL_BIT` — bit de dirección del camino de cuentas |
| `25..29` | `COL_KEY` — **clave de gasto**, cuatro elementos (§90) |
| `29..33` | `COL_ACC_ID` — identidad pública de la cuenta |
| `33` | `COL_BAL` — saldo antes |
| `34` | `COL_BAL_NEW` — saldo después |
| `35` | `COL_NONCE` |
| `36` | `COL_AMT` — importe destruido |
| `37`, `38` | `COL_SUPPLY_OLD`, `COL_SUPPLY_NEW` |
| `39`, `40` | `COL_SBIT`, `COL_SACC` — descomposición binaria del rango |
| `41` | `COL_FBIT` — bit del camino de congelados |

⚠️ **Las columnas `0..24` no tienen constante con nombre.** Son los dos lanes
del estado de Rescue, y **ningún barrido que busque `COL_*` las ve**. §72 —un
fallo de solidez— vivía exactamente ahí.

### 2.2 Filas

| tramo | qué ocurre |
|---|---|
| `0..7` | Absorción de la hoja: capacidad e identidad |
| `7` | `ROW_LEAF_LINK` — enlace de la hoja al camino |
| `15` | `ROW_LEAF_DONE` — hoja computada |
| `15..271` | Ascenso por el árbol de cuentas, 32 niveles × 8 filas |
| `271` | `ROW_ROOT` — raíz alcanzada; **entra la clave** |
| `272..279` | Derivación de la identidad desde la clave |
| `279` | `ROW_PK_DONE` — identidad computada; **se compara** |
| `280..471` | **No-pertenencia al árbol de congelados** |
| `471` | `ROW_FROZEN_ROOT` |

⚠️ El tramo `280..471` **ocupaba filas que estaban libres**. Se añadió porque
una cuenta congelada **podía destruir su dinero**: la liquidación comprobaba
la congelación y la destrucción no.

### 2.3 Los dos lanes NO son simétricos

**Es la propiedad menos evidente del circuito y no está escrita en ningún
sitio del código.** Se deduce cruzando tres bloques separados por cincuenta
líneas:

| | lane A (`0..12`) | lane B (`12..24`) |
|---|---|---|
| Hoja que absorbe | **vieja** (`COL_BAL`) | **nueva** (`COL_BAL_NEW`) |
| Sube por | el mismo camino, mismos hermanos | idem |
| Produce | **raíz vieja** | **raíz nueva** |
| En `ROW_ROOT` recibe | la clave, desde `COL_KEY` | **la misma clave** |
| En `ROW_PK_DONE` | **se compara con `COL_ACC_ID`** | ⚠️ **no se compara** |

⚠️ **Esa asimetría final es correcta y hay que justificarla**: en la fase de
identidad los dos lanes reciben **la misma entrada** —`C_KEY_INPUT` alimenta
ambos desde `COL_KEY`— y **la misma capacidad** —`C_CAP_A` y `C_CAP_B` los
ponen a cero en cada enlace—, así que **computan el mismo digest**. Comprobar
uno basta porque el otro es idéntico **por construcción**.

> **Esa justificación no existía antes de este documento.** Se deducía
> cruzando las líneas 565-566, 606-608 y 615. Es exactamente el tipo de
> razonamiento que §72 no se hizo, y §72 fue un fallo de solidez.

## 3. Las 28 restricciones

`NUM_CONSTRAINTS = C_FBIT_BOOL + 1`.

### 3.1 Hash y camino de Merkle

| ranuras | qué ata | sobre qué filas |
|---|---|---|
| `C_HASH_A`, `C_HASH_B` (12+12) | La ronda de Rescue en cada lane | donde `P_HASH_FLAG` = 1 |
| `C_CAP_A`, `C_CAP_B` (4+4) | Capacidad a **cero** al enlazar | `P_LINK_MERKLE` |
| `C_PLACE_A`, `C_PLACE_B` (4+4) | Colocación según el bit de dirección | `P_LINK_PLACE` |
| `C_SIBLING` (4) | El hermano entra donde toca | `P_LINK_PLACE` |
| `C_BIT_BOOL` (1) | `COL_BIT` ∈ {0,1} | todas |
| `C_LEAF_CAP_A/B` (4+4) | Capacidad de la hoja a cero | `P_LINK_LEAF` |
| `C_LEAF_DIG_A/B` (4+4) | Digest intermedio de la hoja | `P_LINK_LEAF` |

### 3.2 Entrada del testigo

| ranuras | qué ata |
|---|---|
| `C_NONCE` (2) | El nonce entra en las dos hojas |
| `C_INPUT` (10) | **Lane A recibe `COL_BAL`; lane B recibe `COL_BAL_NEW`** — aquí se separan los papeles |
| `C_KEY_INPUT` (8) | Los **cuatro** elementos de la clave, en **ambos** lanes |

### 3.3 Las tres que son el enunciado

| ranuras | qué ata |
|---|---|
| **`C_PK_CHECK`** (4) | **TITULARIDAD**: la identidad derivada = `COL_ACC_ID` |
| **`C_BALANCE`** (1) | `COL_BAL_NEW = COL_BAL − COL_AMT` |
| **`C_SUPPLY`** (1) | `COL_SUPPLY_NEW = COL_SUPPLY_OLD − COL_AMT` |

### 3.4 Constancia y rango

| ranuras | qué ata |
|---|---|
| `C_TRANSPORT` (10) | 10 columnas constantes entre filas: clave (4), saldos, nonce, importe, suministros |
| `C_ID_CONST` (4) | `COL_ACC_ID` constante |
| `C_SBIT_BOOL` (2), `C_FIRST_S` (2), `C_HORNER` (1), `C_SEG_LINK` | Descomposición binaria del importe |
| `C_FROZEN_*`, `C_FBIT_BOOL` | No-pertenencia a congelados |

---

## 4. ⚠️ Qué NO se restringe

**Esta sección es la razón de existir del documento.** Cada `const C_X` del
código dice qué ata; **ninguna dice qué deja libre**, y lo que queda libre es
donde vive un fallo de solidez.

### 4.1 Nada ata `COL_AMT` a ser positivo

`C_BALANCE` exige `BAL_NEW = BAL − AMT` y `C_SUPPLY` lo mismo para el
suministro. **Ninguna exige `AMT > 0`.**

Un `AMT = 0` produce una destrucción vacía: válida, inútil y **sin efecto
sobre el estado salvo el nonce**. No crea dinero.

⚠️ **Y `AMT` negativo no es representable**: el campo es Goldilocks, no hay
signo. Lo que sí es representable es un `AMT` **enorme** que por aritmética
modular «reduzca» el saldo dando la vuelta — **y eso lo impide la
descomposición binaria** (`C_SBIT_BOOL`, `C_HORNER`, `C_SEG_LINK`), que ata
`AMT` a caber en el rango declarado.

**Verificado**: sin esa descomposición, la resta modular permitiría crear
saldo. Con ella, no.

### 4.2 Nada ata el lane B en `ROW_PK_DONE`

Ya explicado en §2.3: es correcto **porque los dos lanes computan lo mismo**,
no porque se compruebe.

⚠️ **Es una garantía condicional**: depende de que `C_KEY_INPUT` y `C_CAP_B`
sigan alimentando el lane B **exactamente igual** que el A. Si alguien cambia
uno de los dos y no el otro, `C_PK_CHECK` **no lo detectará**.

> **Ese es el fallo de §72 en potencia, en este circuito, hoy.** No está
> presente; **está a un cambio de distancia** y nada avisa.

### 4.3 Nada ata `COL_NONCE` a incrementarse

`C_NONCE` lo mete en las dos hojas y `C_TRANSPORT` lo mantiene constante entre
filas. **La hoja vieja y la nueva usan el mismo nonce.**

⚠️ **Consecuencia**: este circuito **no protege contra repetición por sí
mismo**. Lo hace el encadenamiento de raíces —una destrucción cambia la raíz,
así que repetir la prueba presenta una raíz obsoleta—, y **eso exige orden
total**, que un nodo único da y un sistema distribuido no.

Está registrado en `SECURITY.md` §3.4 para la vía de pago; **aplica igual
aquí**.

### 4.4 Nada ata las columnas `0..24` fuera de las filas con selector

Entre `ROW_PK_DONE` (279) y el tramo de congelados, y después de
`ROW_FROZEN_ROOT` (471) hasta el final de la traza (512), **el estado de
Rescue no está restringido**: `P_HASH_FLAG` es cero y los enlaces no
disparan.

**Es correcto** —esas filas no participan del enunciado— pero significa que
un testigo puede poner **cualquier cosa** ahí. No afecta a la validez porque
nada lo lee después.

⚠️ **Si alguna restricción futura leyera esas filas, sería inmediatamente
explotable.** Queda escrito para que quien la añada lo sepa.

### 4.5 El camino de congelados no está atado al índice — **y no hace falta**

`COL_BIT` y `COL_FBIT` son bits de dirección **independientes**, y ninguna
restricción ata que representen la misma posición. La capa pasa el camino
correcto —`self.frozen.path_for(account_index)`— pero **eso es la capa**, y
§73 registra qué pasa cuando una propiedad la impone la capa y no el
circuito.

✅ **Aun así el circuito es seguro, por una garantía IMPLÍCITA**: la raíz de
congelados recomputada se **asevera contra la entrada pública**
(`frozen_root`). Un camino de otra posición **no reconstruye la misma raíz**,
así que la prueba no verifica.

⚠️ **Pero es una garantía por consecuencia, no por restricción** — la misma
forma que §4.2 — y tiene una condición: **depende de que los hermanos de las
dos posiciones difieran**.

En un árbol de congelados **completamente vacío** todos los caminos son
idénticos, y presentar el de otro índice funciona. **Da igual: si nadie está
congelado, la comprobación no protege nada.** El caso que importa —un
congelado presentando el camino de un vecino libre— falla, porque su hoja es
distinta y la raíz no cuadra.

> **La corrección del enunciado importa.** La primera versión de esta sección
> decía «nada lo verifica» y lo dejaba abierto. Lo correcto es: **nada lo
> verifica explícitamente, y la raíz lo verifica implícitamente.** Es
> `AUDITORIA.md` §99.5 aplicado a este documento: un dato correcto sobre el
> objeto equivocado.

---

## 5. Testigos negativos

Los que existen, y qué ataque cubre cada uno:

| test | ataque |
|---|---|
| `third_party_cannot_burn_someone_elses_money` | destruir sin la clave |
| `burning_more_than_the_balance_is_rejected` | saldo negativo |
| `burning_without_updating_supply_is_rejected` | destruir sin reducir el suministro |
| `deflating_supply_beyond_amount_is_rejected` | reducir el suministro de más |
| `a_frozen_account_cannot_burn` | destruir estando congelado |
| `a_forged_frozen_root_is_rejected` | raíz de congelados falsa |
| `wrong_new_root_is_rejected` | raíz nueva que no corresponde |
| `no_constraint_is_vacuous` | mutación: toda restricción dispara |

### ⚠️ Los que faltan

| ataque | estado |
|---|---|
| **Camino de congelados de otro índice** (§4.5) | ⚠️ **sin test, y cerrado por análisis**: la raíz lo impide |
| Lane B divergente del A en la fase de identidad (§4.2) | ❌ **sin test** — no es alcanzable hoy, pero nada lo fija |
| `AMT = 0` | ❌ sin test; **inofensivo**, pero no está declarado |

---

## 6. Qué obligó a preguntar este documento

- **La asimetría de los lanes** (§2.3): estaba en el código, repartida en
  tres bloques, y **no estaba dicha en ninguno**.
- **Que `C_PK_CHECK` es condicional** (§4.2): protege **porque** los lanes
  computan lo mismo, no porque se compruebe. **A un cambio de distancia de
  §72.**
- **Que el camino de congelados no está atado al índice** (§4.5): parecía
  una fuga y **es una garantía implícita** — la raíz lo cierra. Escribir la
  pregunta obligó a encontrar por qué el circuito es seguro, que tampoco
  estaba dicho en ningún sitio.

> Ninguna de las tres se ve leyendo el código de arriba abajo. Las tres
> salieron de preguntar, restricción a restricción: **«si esto ata X, ¿qué
> queda libre?»**
