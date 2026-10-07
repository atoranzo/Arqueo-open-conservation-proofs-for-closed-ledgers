# RFC-0017 — La firma acredita la aritmética: la atadura nativa y el rango a 62 bits

- **Estado:** PROPUESTO (§641) — con sus cuatro etapas construidas en el mismo sello y el canon
  `--sello` VERDE dentro de él. El paso a ACEPTADO es del autor: la regla 4 del PROCESO pide la
  spec, el OpenRPC (no se mueve: ningún método ni ningún campo cambia), los vectores (ninguno nuevo
  bajo `spec/vectors/`: lo que entra son tests negativos en los crates) y las suites; todo eso está
  en el §641, y la decisión no.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§641): midió, propuso,
  aplicó, corrió el canon y commiteó, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-01
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube**: ni un método ni un campo del cable
  cambian. Cambia lo que se ACEPTA: un importe, un saldo o un suministro en la ventana alta de 63
  bits (`[2^62, 2^63)`) dejan de pasar; el techo sano del campo baja de `2^63 - 1` a `2^62 - 1`.
  ⚠️ **Corregido en el §699**: tampoco pasan las pruebas honradas de antes. El selector periódico de
  los diecisiete AIR cambia, y una prueba de envío del §640 no verifica con el AIR del §641, ni al
  revés (medido con el envío); `zkssl/0.4` no lo dice.
- **Asiento(s) de AUDITORIA:** §641.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la capa no confía | `validate_send` ata `amount == pi.amount`, el estado del remitente a la hoja del árbol y `pi.amount <= límite`, y resta con `checked_sub`; el cobro ata `notice.amount == pi.amount` y suma con `checked_add`; la quema deriva el suministro nativo (`checked_sub`) en vez de copiar `pi.supply_new`, y ata su estado a la hoja | NO | sellada — §641 |
| E2 — el rango a 62 bits en los AIR de la capa | los diecisiete circuitos de `stark-experiment` fuerzan a cero **dos** bits altos (63 y 62) de cada segmento de rango, no uno: el rango demostrado baja a `[0, 2^62)` y una resta que envuelve en Goldilocks ya no cabe. `MAX_VALUE` de `compliance_circuit`, `double_entry` y `range_check` a `2^62 - 1` | NO | sellada — §641 |
| E3 — la auditoría ata la identidad y acota la banda | `circuit_audit::get_assertions` ata `COL_ID` (la cuenta del testigo) al `public_id` público, y `verify_audit` exige `lower <= upper <= 2^62 - 1` antes de tocar la prueba | NO | sellada — §641 |
| E4 — el saldo del cliente, canónico en el cable | `zk-ssl-wire` lee `balance` con la regla de RFC-0015 (`u64` canónico, `< p`) en los dos DTO de estado, antes de que la capa lo sume o reste | NO | sellada — §641 |

## Motivación

El núcleo prueba `saldo - importe >= 0` (y las bandas del tipo `inferior <= saldo <= superior`)
descomponiendo la **diferencia** en bits y comprobando que cabe en un rango. Sobre el campo de
Goldilocks, `p = 2^64 - 2^32 + 1`, una resta `a - b` con `b > a` no da un número negativo: da
`p - (b - a)`, un elemento del campo. Si ese elemento cabe en el rango que el circuito comprueba, la
prueba acredita una resta que **nunca ocurrió**.

El rango era de **63 bits** (`< 2^63`). Una resta envuelta vale `p - d`, con `d = b - a` el déficit.
Para que `p - d < 2^63` basta `d > p - 2^63 = 2^63 - 2^32 + 1`. Y como los operandos también se
acotan a 63 bits, `d` puede llegar hasta casi `2^63`: existe una ventana de unos `2^32` déficits —los
mayores que `2^63 - 2^32 + 1`— cuya resta envuelta **sí** pasa el rango de 63 bits. Medido en la
sesión: un envío con saldo `0` e importe en esa ventana se probaba y la capa escribía un saldo
nuevo enorme, acuñando valor de la nada; el cobro repetía el patrón sumando al saldo que manda el
cliente; y la quema tomaba el suministro de la prueba.

La cuenta que lo cierra: si los operandos caben en **62 bits** (`< 2^62`), un déficit real cumple
`d < 2^62`, luego una resta envuelta vale `p - d > p - 2^62`, que es mayor que `2^62` y **no cabe**
en un rango de 62 bits. `2 * 2^62 < p`, así que no hay solape. El rango de 62 bits es sólido; el de
63 no lo era.

## Lo medido, antes de tocar nada

Con el probador y el verificador reales (perfil release), sobre la capa:

| caso | antes | después |
|---|---|---|
| envío: saldo 0, límite 500 000, importe en la ventana alta de 63 bits | `Ok(())` — valor acuñado | rechazado |
| envío: importe de control 500 001 | rechazado | rechazado |
| quema: saldo `1e6`, suministro `1e8`, importe `2^63 - 1` | `Ok` | rechazado |
| auditoría: banda `[2^63-1, 2^63-1]` sobre saldo 0 | `Ok` | rechazado (cota de banda) |
| auditoría: `public_id` ajeno con un probador propio | `Ok` | rechazado (atadura de identidad) |

## Lo que hace

1. **La capa no confía** (E1). `validate_send`: el parámetro `amount` ha de ser el `pi.amount`
   PROBADO; el estado del remitente, la hoja que el árbol guarda; `pi.amount <= límite`; y el saldo
   nuevo sale de `checked_sub`, que rechaza en vez de envolver. El cobro ata `notice.amount` al
   `pi.amount` y usa `checked_add`. La quema deriva el suministro nuevo con `checked_sub` del
   suministro vigente y exige que la prueba acredite EXACTAMENTE ese valor, en lugar de copiar
   `pi.supply_new`; y ata su estado a la hoja del árbol, como el cobro.
2. **El rango a 62 bits** (E2). En los diecisiete circuitos de `stark-experiment`, el selector
   periódico de la primera fila de cada segmento (`first_s`) pasa a cubrir las filas 0 **y 1**: la
   restricción `first_s * sbit` fuerza a cero el bit 63 y el bit 62. No cambia ni el número de
   restricciones, ni los grados, ni el conteo de aserciones, ni las columnas. `MAX_VALUE` de
   `compliance_circuit`, `double_entry` y `range_check` baja a `2^62 - 1` (en `circuit_audit` ya lo
   era), y `range_check` fija además el bit 62 a cero con una aserción (5 -> 6).
3. **La auditoría** (E3). `circuit_audit::get_assertions` ata las cuatro ranuras de `COL_ID` en la
   fila 0 al `public_id` de las entradas públicas (17 -> 21 aserciones): el probador acreditaba SU
   cuenta y declaraba el `public_id` de otra, y el verificador atribuía la banda a la víctima.
   `verify_audit` exige `lower <= upper <= 2^62 - 1` antes de parsear la prueba.
4. **El cable** (E4). `zk-ssl-wire` lee el `balance` de `ClientStateDto` y de `AccountViewDto` con la
   regla canónica de RFC-0015 (reutilizando `element_from_bytes`): un saldo escrito como `b + p` se
   rechaza en el cable, antes de que la capa lo sume o reste.

## Decisiones (REVERSIBLES)

- **D-1: dos defensas, no una.** El rango a 62 bits cierra el hueco en el circuito; la atadura nativa
  lo cierra en la capa, y además tapa la vía de llamar al `apply` con una traza propia sin pasar por
  el probador de la casa. Ninguna sola basta: el circuito protege al tercero que sólo tiene la
  prueba; la capa protege al nodo.
- **D-2: 62 y no menos.** `2 * 2^62 < p` es la cota exacta que elimina el solape; 62 bits es el
  máximo rango sólido sobre Goldilocks para una resta. `2^62 - 1` sigue siendo un techo de unos
  `4,6 x 10^18` unidades mínimas, muy por encima de cualquier importe real.
- **D-3: el kit YA estaba.** Los AIR del kit (`zk-ssl-air`: `banda`, `cobro_pendiente`) exigen
  `comprobar_enunciado` en el verificador —`lower, upper <= MAX_VALOR = 2^62 - 1` y `lower <=
  upper`— antes de verificar el STARK, así que el tercero que verifica con el kit ya estaba a salvo
  del wraparound por una cota NATIVA. E3 lleva a `circuit_audit` y a `verify_audit` al mismo nivel
  que el kit ya tenía. No se toca ningún AIR del kit, y por tanto ningún vector portable.
- **D-4: el RFC queda PROPUESTO**; aceptarlo es del autor.

## Lo que NO hace

- No sube el cable ni toca el OpenRPC. No mueve un KAT ni una cabeza firmada.
- No añade ni regenera un vector de `spec/vectors/`: lo que entra son tres tests negativos en los
  crates (el `public_id` ajeno en `stark-experiment`, la cota de banda en `zk-ssl`, el saldo no
  canónico en `zk-ssl-wire`).
- No toca los AIR del kit (`zk-ssl-air`), ya protegidos por `comprobar_enunciado` (D-3), ni sus
  vectores portables (`edad`, `pendiente`, `pago`, `prenda`), que son capturas no regenerables en
  `--sello`.
- No corre `--bancos` ni `--completo`: sólo `--sello`.

## Lo que corrige de RFC-0015

El §631 (RFC-0015) cerró con «ningún veredicto de un tercero depende» del lector de `u64` del cable
sin la regla canónica. E4 lo cierra para el saldo del cliente: el estado que el titular aporta se lee
canónico antes de que la capa lo use, de modo que un saldo escrito como `b + p` ya no descuadra la
conservación del suministro al reabrir. Queda nombrado en el §641.
