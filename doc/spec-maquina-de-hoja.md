# B13/B14 · Paso 1.5 — Spec de la MÁQUINA DE HOJA (leer antes de tocar)

Existe por §72: una spec escrita caza el bug de carril leyéndola, y los
diez AIR son diez oportunidades de ese bug. Referencia: `circuit_send`
(el piloto). Anatomía verificada en código, no de memoria.

## 1. La máquina COMO ES (send)

**Geometría**: `TRACE_LENGTH = 1024`, `TRACE_WIDTH = 52`. Dos carriles
Rescue de `STATE_WIDTH = 12` (capacidad 0..4, digest 4..8, rate 8..12):
carril A en columnas `0..12`, carril B en `LANE_B = 12..24`.

**Los carriles NO son redundancia: son estado viejo y nuevo.** A computa
la hoja VIEJA y prueba su pertenencia; B computa la NUEVA y construye el
camino posterior. `C_SIBLING` ata ambos al MISMO camino (mismos hermanos,
mismo bit); `C_PLACE_B` coloca el digest nuevo en la subida de B.

**La cadena espeja `native_leaf` exactamente** — dos merges:
`merge(merge(public_id, [bal,0,0,0]), [nonce,0,0,0])`.
- `first_row`: `C_INPUT` inyecta en el digest de cada carril el
  `COL_ACC_ID` (la misma identidad en A y B) y en el rate `[8]` el
  balance — **`COL_BAL` en A, `COL_BAL_NEW` en B**: aquí nace la
  diferencia entre carriles que `C_BALANCE` audita.
- `link_leaf` (columna periódica, 1 en `ROW_LEAF_LINK`): cose el segundo
  merge. `C_LEAF_CAP_{A,B}`: capacidad de `next` a cero. `C_LEAF_DIG_{A,B}`:
  el digest se arrastra (`next[4+i] == current[4+i]` — la salida del
  primer merge es la entrada izquierda del segundo). `C_NONCE` (×2): el
  rate `[8]` de `next` := `COL_NONCE`, **en ambos carriles** (§92.2:
  atar solo un carril deja el otro libre).
- `C_KEY_INPUT` con `sel_root`: los CUATRO limbos de la clave, ambos
  carriles (§92.2 otra vez, ya cazado una vez).

**⚠️ Cuestión a verificar en el paso 2** (honestidad de esta spec): los
limbos `9..11` del rate en el merge del nonce (`as_digest(nonce)` =
`[nonce,0,0,0]`) — localizar QUÉ restricción los ata a cero. Si ninguna,
es un hallazgo previo al salt y se cierra primero.

## 2. Delta SALT (la envoltura: un merge más)

`native_leaf_salted = merge(native_leaf, leaf_salt)` → **un tercer ciclo
de merge por carril**, tras el del nonce:
- **+4 columnas testigo** `COL_SALT..+3` (52 → 56). UN solo salt
  compartido por ambos carriles: el salt NO cambia en send/claim/burn/
  mint/transfer (se preserva, 1a). En recovery el record COPIA el salt
  viejo (costura 52) → también un solo salt allí. Ningún circuito de hoy
  necesita dos salts. (El día que la 52 rote el salt, ese circuito
  necesitará `COL_SALT_NEW` — fuera de alcance, anotado.)
- **+1 selector periódico** `link_salt`, 1 en `ROW_SALT_LINK =
  ROW_LEAF_LINK + CYCLE`. Todos los `ROW_*` posteriores se corren
  `+CYCLE` (8 filas). Coste: +8 filas por circuito de hoja.
- **Restricciones nuevas** (×2 carriles, las seis):
  `C_SALT_CAP_{A,B}`: capacidad a cero. `C_SALT_DIG_{A,B}`: digest
  arrastrado. `C_SALT_IN_{A,B}`: **los CUATRO limbos** del rate :=
  `COL_SALT..+3` — el salt es un digest completo, no un escalar; atar
  solo `[8]` sería el bug de §92.2 en su forma nueva.
- **Uniformidad** (D-arquitectura): cuentas legacy → salt testigo =
  ceros, MISMA máquina. Sin rama condicional en el AIR.
- La entrada del camino de Merkle pasa a ser la salida del TERCER merge.
  El enlace hoja→camino se corre con los `ROW_*`.

## 3. Delta FROZEN-32 (§137)

Los caminos de congelación crecen de 24 a 32 niveles: **+8 ciclos = +64
filas** en cada circuito que verifica camino frozen (send, claim, burn,
freeze, frozen_climb). Presupuesto por circuito: TABLA A RELLENAR en los
pasos 2-3 midiendo filas usadas hoy — **no se asume que 1024 alcanza**.

| circuito | hoy (fila final) | + salt | + frozen-32 | holgura en 1024 | veredicto |
|---|---|---|---|---|---|
| send (PILOTO, §143) | 743 | 751 | **815** | **208** (26 ciclos) | **CABE** |
| claim (§144) | 743 | 751 | **815** | **208** (26 ciclos) | **CABE** |
| burn (§145) | 471 | 479 | **543** | **480** (60 ciclos, en SU 1024) | **NO CABE en 512 → TRACE propia 1024** |
| mint (§146) | 311 | 319 | — (sin frozen) | **192** (24 ciclos, en su 512) | **CABE** |
| audit (§147) | 279 | 287 | — (sin frozen) | **224** (28 ciclos, en su 512) | **CABE** |
| mint_climb (§148) | 271 | 279 | — (sin frozen) | **232** (29 ciclos, en su 512) | **CABE** |

(Primera fila: medida por el piloto en el gemelo; el resto se rellena
circuito a circuito en el paso 3.)

Si un circuito desborda: `TRACE_LENGTH` propio a 2048 (coste ~2× en su
prueba, medido en el paso 5) o compactación de fases — decisión POR
circuito, con el dato delante.

## 4. La obligación de MUTACIÓN (por circuito, innegociable)

Cada AIR migrado trae al menos DOS sabotajes que deben RECHAZARSE:
(a) un limbo del salt testigo alterado → `C_SALT_IN` dispara;
(b) el tercer merge omitido (hoja sin envolver entra al camino) → el
    corrimiento de enlace dispara.
Si un sabotaje NO rompe la prueba, la restricción es decorativa (§72).

## 5. Checklist por circuito (pasos 2-3)

| circuito        | hoja+salt | frozen-32 | nota |
|-----------------|-----------|-----------|------|
| send (PILOTO)   | sí        | sí        | referencia de esta spec |
| claim           | sí        | sí        | |
| burn            | sí        | sí        | |
| mint            | sí        | —         | |
| audit           | sí        | —         | |
| recovery        | sí        | —         | salt único (copia, costura 52) |
| recovery_climb  | sí        | —         | |
| mint_climb      | sí        | —         | |
| freeze          | —         | sí        | solo profundidad |
| frozen_climb    | —         | sí        | solo profundidad |

Orden: piloto send completo (salt + frozen-32 + mutaciones + su test
nativo↔circuito), luego réplica uno a uno. Cada circuito: leer su región
de hoja ANTES (las variantes existen: mint no tiene carril de resta,
audit no muta estado), aplicar, mutar, suite entera.
