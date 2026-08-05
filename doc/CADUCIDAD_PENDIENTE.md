<!--
  CADUCIDAD_PENDIENTE — el reembolso del emisor tras T, con doble cerrojo.
  Documento de diseño. Cierra la única deuda viva que §177 dejó anotada:
  «la caducidad del pendiente inmovilizado». Hermano de ANCLAJE_EXTERNO.md.
  No toca el compromiso, no toca claim, no toca los discriminantes de §173.
-->

# CADUCIDAD_PENDIENTE — que el dinero en tránsito tenga reloj

## 0. El problema, con su acta

Dos-fases inmoviliza: `send` debita al emisor y crea el compromiso
`P = H(H(id_receptor, salt), importe)`; solo el `claim` del receptor lo
cobra. Si el receptor nunca cobra —clave perdida, cuenta congelada
(§16.2), o simplemente nunca—, los fondos quedan quietos **para
siempre**: no existe hoy operación de cancelar, caducar ni devolver. Es
**coste DECLARADO** (§29-§30, entrada 12) y menos limbo que la vía de un
paso que sustituyó (§7/§36) — pero §177 lo señaló como **el único frente
donde el modelo de notas ofrecería algo** (time-locks) que aquí falta. Este
documento diseña ese algo **sobre el modelo actual**, como §177 prescribió.

## 1. El ataque que mata al diseño ingenuo — y el doble cerrojo

**Refund-por-apertura es robable.** Hoy, filtrar el aviso
(`position, salt, amount`) es inocuo: `claim` ata la clave del receptor al
`id_receptor` del compromiso (§39.1, `C_PK_CHECK`) — conocer el aviso no
cobra. Un reembolso que solo exigiera *conocer la apertura de P* rompería
eso: el ladrón-con-aviso reembolsaría **a su propia cuenta**. El diseño
cierra la puerta dos veces:

1. **El destino lo fijan los registros, no el probador.** En `apply_send`
   la capa ya recibe `sender_index`; se anota
   `pending_meta[pos] = { sender_index, born_seq }` junto al
   `pending_amounts` que **ya existe** (con la misma nota de honestidad:
   metadato del operador, que ya conoce al emisor — §21/§129; los
   terceros no lo ven, los discriminantes de §173 quedan intactos). El
   reembolso acredita a `meta.sender_index` **pruebe quien pruebe**: el
   ladrón, en el peor caso, le paga el trámite al emisor.

2. **Solo el emisor puede fabricar los materiales.** El crédito exige la
   subida de SU hoja —preimagen completa: `public_id`, saldo, nonce y
   `leaf_salt`, que deriva de SU clave de gasto (§117)—. El aviso no la
   contiene. Doble cerrojo: sin la clave no hay materiales; con
   materiales ajenos no hay destino.

## 2. El mecanismo

**La operación**, de una parte (como `claim`):

```text
refund(pos, apertura, subida_de_crédito)
  CAPA verifica:  seq_actual − meta[pos].born_seq ≥ T     ← el reloj
                  hoja[pos] == P(apertura)                 ← sigue viva
  CIRCUITO refund: apertura de P  +  camino P → vacía      ← nuevo, pequeño
  CIRCUITO crédito: mint_climb EXISTENTE, tal cual         ← reuso
                  hoja(saldo) → hoja(saldo + importe)
                  sobre la cuenta meta[pos].sender_index   ← destino fijado
  COMMIT: lote con {hoja emisor, hoja pendiente, meta}     ← la lección §169
```

- **`circuit_refund`** es clase-claim-ligera: prueba la apertura del
  compromiso (la misma reconstrucción `C_PEND_IN` que claim ya hace) y el
  vaciado del camino de pendientes. **Sin PK-check** (el destino ya está
  fijado), **sin nullifier** (la hoja vacía es su propio anti-replay: el
  segundo intento no encuentra P), **sin frozen-check** (devolver no es
  cobrar). El guardián de layout pasará de 26 a **27 circuitos**.
- **El crédito reusa `mint_climb`** sin tocar una coma: es exactamente
  «hoja(b) → hoja(b+importe), mismo pid/nonce/salt».
- **El reloj**: `born_seq = epoch_head().seq` al aplicar el send. `T` es
  **línea sistémica declarada** —familia de `N_max` (§121) y `M`
  (§174)—: se elige, se publica, se mide. Cuando las cabezas firmadas
  existan (§115), `T` hereda su reloj sin cambiar el diseño; estirar el
  latido para esquivar `T` ya es evidencia oponible (§121.4).

## 3. La carrera, declarada

Tras `T`, **claim y refund compiten y el primero en aplicarse gana**: ambos
vacían la posición, el segundo choca con hoja-vacía y rebota. No es un
defecto sino la semántica elegida: pasado el plazo, el derecho del receptor
deja de ser exclusivo. El receptor diligente cobra antes de `T`; el emisor
no recupera antes de `T`. **Un solo parámetro parte el tiempo en dos
derechos.**

## 4. Los pendientes de EMISIÓN se des-emiten

`mint_to_pending` crea pendientes **sin emisor-cuenta** (el dinero nace
ahí). Su caducidad no reembolsa: **des-emite** —la hoja se vacía y
`total_supply` baja en el importe—, con la simetría limpia de que *la
emisión no cobrada deja de existir* (los custodios pueden re-emitir si
procede, por la vía delegada). Misma compuerta `T`, sin subida de crédito,
sin circuito extra: la apertura del compromiso basta y el destino es la
nada contable.

## 5. Lo que este diseño NO toca — con testigos

- **El compromiso**: idéntico. `el_commitment_no_revela_al_emisor` (§173)
  sigue en verde — el emisor entra en los REGISTROS del operador (que ya
  lo conocía), jamás en el árbol.
- **`claim`**: ni una constante. El receptor pre-`T` cobra exactamente
  como hoy.
- **El aviso**: mismo `PendingNotice`, misma inocuidad al filtrarse — el
  doble cerrojo lo garantiza.
- **La inenlazabilidad**: el salt sigue siendo el portador declarado
  (§173); `refund` no añade correlación visible a terceros.

## 6. Lo que NO resuelve

- **El emisor con clave perdida** tampoco recupera (no puede fabricar la
  subida). Su caso —y el de operaciones huérfanas de toda clave— es la
  **fase 2**: barrido a remanente con ventana de reclamos, que es
  decisión política de la familia **B18.3** (§87/§88) y se discute con esa
  familia, no aquí.
- **El operador puede fabricar materiales de refund** (tiene los
  registros). Es mantenimiento benigno —el destino sigue fijado al
  emisor— y se declara: un operador solo puede *acelerar* devoluciones,
  nunca desviarlas.
- **`T` es un juicio**, no un teorema: corto castiga receptores lentos,
  largo alarga la inmovilización. Se declara y se revisa con datos, como
  `N_max`.

## 7. Coste y primer paso medible

Una estructura (`pending_meta`, junto a `pending_amounts` y persistida en
el lote), una compuerta de capa, **un circuito pequeño** (clase claim-sin-
identidad; el guardián dirá 27), el reuso de `mint_climb`, y los tests: el
refund legítimo tras `T`, el rechazo pre-`T`, la carrera, el ladrón-con-
aviso (rebotado por destino y por materiales), el replay (hoja vacía), la
des-emisión del mint-pendiente, y la persistencia del meta a través del
reinicio — la lección del commit huérfano (§169) escrita como test desde
el primer día. **Primer paso**: `circuit_refund` con su layout en el
guardián y sus discriminantes, antes de tocar la capa. Es trabajo de
circuito: **sesión propia**, como manda la casa para todo lo que estrena
restricciones.
