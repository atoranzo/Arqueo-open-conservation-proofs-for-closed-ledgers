# RFC-0017 — La firma acredita la aritmética: la atadura nativa y el rango a 62 bits

- **Estado:** PROPUESTO (§641) — con sus cuatro etapas construidas en el mismo sello y el canon
  `--sello` VERDE dentro de él. El paso a ACEPTADO es del autor: la regla 4 del PROCESO pide la
  spec, el OpenRPC (no se mueve: ningún método ni ningún campo cambia), los vectores (ninguno nuevo
  bajo `spec/vectors/`: lo que entra son tests negativos en los crates) y las suites; todo eso está
  en el §641, y la decisión no. ⚠️ §826: no todo. Su E2 rompió las pruebas que viajan (§699), y de
  una ruptura del cable la regla 4 pide vectores re-emitidos o nuevos bajo la versión nueva, que no
  hay todavía: si nacen, nacen con el corte `zkssl/0.5`, que paga la ruptura, y que nazcan para las
  pruebas de la capa no está decidido (ver «Compatibilidad»).
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
  ⚠️ §826: con la política del RFC-0021, eso es romper la superficie S2, las pruebas que viajan, sin
  subir la versión; lo paga el corte `zkssl/0.5` del RFC-0018 (ver «Compatibilidad»).
- **Asiento(s) de AUDITORIA:** §641 (la propuesta y sus cuatro etapas); §699 (lo que la E2 cambió en
  las pruebas que viajan, medido, y la corrección de la cabecera); §719 (completa la E2: el segmento
  del tope de la emisión a pendiente y `solvency`, a 62 bits); §723 (la corrección de la D-3: la
  banda y el cobro pendiente del kit, a 62 bits con su v2); §778 (las citas heredadas del
  RFC-0015 y del §631, corregidas); §826 (su compatibilidad, escrita, y su sección de seguridad).
  ⚠️ §826: decía «§641.».

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la capa no confía | `validate_send` ata `amount == pi.amount`, el estado del remitente a la hoja del árbol y `pi.amount <= límite`, y resta con `checked_sub`; el cobro ata `notice.amount == pi.amount` y suma con `checked_add`; la quema deriva el suministro nativo (`checked_sub`) en vez de copiar `pi.supply_new`, y ata su estado a la hoja | NO | sellada — §641 |
| E2 — el rango a 62 bits en los AIR de la capa | los diecisiete circuitos de `stark-experiment` fuerzan a cero **dos** bits altos (63 y 62) de cada segmento de rango, no uno: el rango demostrado baja a `[0, 2^62)` y una resta que envuelve en Goldilocks ya no cabe. `MAX_VALUE` de `compliance_circuit`, `double_entry` y `range_check` a `2^62 - 1` | **sí**: S2, las pruebas que viajan; una de envío del §640 no verifica con el AIR del §641, ni al revés (§699). ⚠️ §826: decía «NO» | sellada — §641; completada — §719. ⚠️ **Completado en el §719**: el segmento del tope de `MintPendingAir` y de su subida, `MintPendingClimbAir`, tiene su propio selector, `cap_first`, y seguía con una sola fila: 63 bits. `solvency`, fuera de los diecisiete, también. El §719 los baja a 62 bits. Como el resto de la E2, una prueba de esos AIR hecha antes no verifica después; pero ninguno de los tres viaja, con el criterio de la E3 (ver «Compatibilidad»): ningún método del OpenRPC los lleva, y `MintPendingClimbAir`, el único que juzga la capa, sólo lo verifica `apply_mint_pending_delegated`, API de la capa, fuera de la promesa. No rompe S2 |
| E3 — la auditoría ata la identidad y acota la banda | `circuit_audit::get_assertions` ata `COL_ID` (la cuenta del testigo) al `public_id` público, y `verify_audit` exige `lower <= upper <= 2^62 - 1` antes de tocar la prueba | NO | sellada — §641 |
| E4 — el saldo del cliente, canónico en el cable | `zk-ssl-wire` lee `balance` con la regla de RFC-0016 (`u64` canónico, `< p`) en los dos DTO de estado, antes de que la capa lo sume o reste | NO | sellada — §641 |

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
   ⚠️ **Completado en el §719**: dos sitios de `stark-experiment` no tenían la segunda fila. El
   segmento del tope de `MintPendingAir` y de su subida, `MintPendingClimbAir`, descompone
   `tope - suministro_nuevo` con su propio selector, `cap_first`; y el rango de `solvency` tenía
   `first[0]` y no `first[1]`. Seguían en 63 bits: en la subida, con un tope de `1e8`, un
   suministro nuevo «negativo» (`tope - margen + p`, con un margen de `2^62 + 5`) verificaba. El
   §719 pone la segunda fila en los tres AIR (`cap_first[CAP_START + 1]` y `first[1]`), baja
   `EFFECTIVE_BITS` a 62 y `MAX_VALUE` a `2^62 - 1` en `solvency`, y da a cada segmento de rango,
   el del tope incluido, un test que mira sus dos bits altos.
3. **La auditoría** (E3). `circuit_audit::get_assertions` ata las cuatro ranuras de `COL_ID` en la
   fila 0 al `public_id` de las entradas públicas (17 -> 21 aserciones): el probador acreditaba SU
   cuenta y declaraba el `public_id` de otra, y el verificador atribuía la banda a la víctima.
   `verify_audit` exige `lower <= upper <= 2^62 - 1` antes de parsear la prueba.
4. **El cable** (E4). `zk-ssl-wire` lee el `balance` de `ClientStateDto` y de `AccountViewDto` con la
   regla canónica de RFC-0016 (reutilizando `element_from_bytes`): un saldo escrito como `b + p` se
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

  > ⚠️ **Corrección del §723 (la ventana del saldo privado).** Para la banda y el cobro
  > pendiente, D-3 es falsa. `comprobar_enunciado` acota los límites PÚBLICOS; el saldo de la banda
  > y el importe del cobro son PRIVADOS, y sólo los acotaba el segmento 0 de su rango, que en el kit
  > seguía en 63 bits (`primera_s[seg * LARGO_SEGMENTO] = uno`). Con un superior `u` por debajo de
  > `2^64 - 1 - p = 2^32 - 2`, un valor de la ventana `[2^62, 2^63)` pasaba la banda: medido con el
  > juez del kit, una hoja con saldo `2^63 - 2^32 + 2` probaba «saldo en `[0, 0]`» y verificaba, y
  > con `4_999_999` más, `[0, 4_999_999]`; los controles con `2^62 + 1` no verificaban. Una
  > transición honesta no deja esa hoja, y la capa no la firma (su tope rechaza el saldo al cargar
  > el almacén y al importar la instantánea), pero un operador que firma una raíz a mano obtenía un
  > VERDE falso de `InsufficientBalance` ante quien sólo comprueba la cabeza y la banda. El cobro
  > pendiente tenía la misma forma en `verificar` con un superior pequeño; `verificar_contra_cabeza`
  > fija el superior en `2^62 - 1`, y ahí la ventana está vacía. El pago en curso es sólido. Lo
  > cierra una **v2** de `BandaAir` y de `CobroPendienteAir`, con los tres segmentos a 62 bits como
  > las AIR vivas de E2 (`primera_s[seg * LARGO_SEGMENTO + 1] = uno`), en el kit y en la segunda
  > implementación. Las pruebas de la v1 ya emitidas se juzgan por su superior: desde `2^32 - 2`
  > dicen lo que parecen y siguen verificando; por debajo, el mando da el cuarto estado,
  > «DECLARADA, NO PROBADA» (salida 3). La única prueba de la v1 bajo la cota que ya se publicó,
  > la de `rechazo/saldo-insuficiente.json` (capturada de un nodo de la casa), sigue VERDE: el juez
  > la reconoce por la huella de su contenido y su enunciado, en una lista cerrada que no crece, y
  > ningún vector cambia de veredicto. El texto de arriba se conserva como se escribió.
- **D-4: el RFC queda PROPUESTO**; aceptarlo es del autor.

## Compatibilidad

⚠️ §826: esta sección no existía, y la regla 2 del PROCESO la pide. Lo que el RFC decía de su
compatibilidad estaba en su cabecera («no sube») y en «Lo que NO hace» («No sube el cable»), y el
§699 corrigió la cabecera con lo que midió.

**Rompe el cable, y no subió la versión.** La E2 cambió el selector periódico `first_s` de los
diecisiete AIR de la capa, y con él los bytes de las pruebas que viajan, la superficie S2 de la D-A
del RFC-0021: una prueba de envío del §640 no verifica con el AIR del §641, ni al revés. Lo midió el
§699 con el envío, entre nodos y clientes de seis puntos de la historia que dicen todos `zkssl/0.4`
(`spec/RPC.md`, «Versionado», en «Notas operativas»). Por la regla 2 del PROCESO eso sube la
versión, y por la R1 y la R3 del RFC-0021 también; el cable siguió en `zkssl/0.4` porque la
conformidad de `0.4` no compara bytes de prueba (RFC-0009, D-AI). Lo que hizo el §699 fue escribir el
aviso de lo que ya estaba roto: la línea bajo el título de `spec/RPC.md`, la descripción del OpenRPC
y la fila «Estado» de los README.

**Lo paga el corte.** La ruptura se paga en el corte `zkssl/0.5` del RFC-0018, junto con las de la
E1 (§680) y la E2 (§684) de aquel RFC, que cambiaron otras pruebas del cable bajo la misma versión:
una ruptura para varias razones (R4 del RFC-0021). Los vectores que la regla 4 pide de una ruptura
del cable —re-emitidos o nuevos bajo la versión nueva— no están decididos. Nacerían en el corte si
la E3b del RFC-0021 guarda como vector la prueba positiva de cada familia de la capa, como propone
su D-D, juzgada en cada canon por el verificador del nodo; el del kit no la juzga, porque hoy el kit
no verifica pruebas de envío, cobro, quema ni umbral (`spec/rfc/0018-el-tren-0-5.md:110`) y este
RFC no tocó ningún AIR del kit (D-3). Eso cambia si la versión fuerte de `ProofFailed` y
`VerificationFailed`, que la E3 del RFC-0019 lleva al mismo corte, le da al kit un verificador de
las pruebas de la capa (lo dice la tabla de etapas de aquel RFC). Hoy la D-G del RFC-0018 prevé
positivas sólo de las cinco familias del kit (`spec/rfc/0018-el-tren-0-5.md:399`), y en las
familias que sólo juzga el nodo sus casos van como tests de la capa, no como vectores
(`:424-426`); la E3b remite a ella para esas familias. Hasta el corte no cambia ninguno de
`spec/vectors/`, y este RFC sigue PROPUESTO: la regla 4 pide vectores para aceptarlo, y cuáles
cubren su ruptura lo decide el corte.

**Lo demás deja de aceptar, sin cambiar lo que se escribe.** La E1 y la E4 no cambian un método,
un campo ni la escritura de un valor: dejan de aceptar lo que acuñaba valor o descuadraba la
conservación —un importe, un saldo o un suministro en la ventana alta de 63 bits, que ningún
importe real alcanza (D-2: el techo sano baja a `2^62 - 1`); un estado que no es la hoja del
árbol; un saldo escrito como `b + p`, que ningún productor de la casa escribe—. La E3 cambia la AIR
de la auditoría, que no viaja: ningún método del OpenRPC la lleva, y la API de la capa está fuera
de la promesa (D-A del RFC-0021). Ni un método, ni un campo, ni un KAT, ni una cabeza firmada se
mueven, y los AIR del kit no se tocan (D-3), así que sus vectores portables tampoco.
⚠️ §719: lo que el §719 completa de la E2 —el segmento del tope de `MintPendingAir` y de su subida,
`MintPendingClimbAir`, y el rango de `solvency`, a 62 bits— cambia tres AIR que tampoco viajan, por
lo mismo que la de la auditoría: ningún método del OpenRPC los lleva y el nodo no verifica ninguno.
`MintPendingClimbAir` sólo lo verifica `apply_mint_pending_delegated`, API de la capa, y la entrada
`MintToPending` del registro lleva el digest del sello de su autorización, no el de esa prueba
(`spec/RPC.md`, la tabla de `proofDigest`); `MintPendingAir` y `solvency` no los verifica ningún
juez de producción. El censo de `crates/zk-ssl/src/instrumento_revela.rs` marca la subida, como la
auditoría, porque la verifica la capa, y la D-G del RFC-0018, en propuesta, la cuenta entre las
catorce familias que juzga la capa, que el corte versionaría con su marca v2; ni lo uno ni lo otro
las hace viajar bajo `zkssl/0.4`. No rompe S1 ni S2, y no se suma a las rupturas de `zkssl/0.4` que
declaran `spec/RPC.md` y los README.
⚠️ §723: «este RFC no tocó ningún AIR del kit (D-3)», arriba, es cierto del RFC, y la D-3 que lo
justificaba era falsa para la banda y el cobro pendiente (ver su corrección): el valor que acotan
es privado, y en el kit su rango seguía en 63 bits. El §723 los baja a 62 bits con una v2 de las
dos AIR, en el juez del kit, en los probadores y en la segunda implementación. Ésas sí viajan: la
banda, en el sobre de rechazo `InsufficientBalance` que escribe el nodo (`--prueba-rechazo`), y el
cobro, en el sobre del cobro pendiente que arma el cliente; las dos las juzga el kit. La v2 cambia
el selector de una columna periódica, y con él los bytes de esas pruebas: la 0.4.3 y las anteriores
no verifican una prueba de la v2. Es una ruptura de S2 sin subir la versión, ya bajo la promesa, que
rige desde el §799, y la paga el mismo corte `zkssl/0.5`, como la de la E2. El kit sigue leyendo la
v1 ya emitida, por su superior o, la única ya publicada bajo la cota, por su huella en una lista
cerrada, y por debajo da el cuarto estado, salida 3: un arreglo de seguridad que cambia un
veredicto dentro de `0.N.x`, con su aviso. Ningún vector publicado cambia de veredicto ni se muda;
entran once nuevos, con sus líneas al final de los manifiestos de `rechazo/`, `completitud/` y
`pendiente/`. Se suma a las rupturas de `zkssl/0.4` que declaran `spec/RPC.md`, `spec/README.md` y
los README; la descripción del OpenRPC habla de las pruebas que un nodo acepta por el cable, y éstas
no las acepta ninguno, así que no la toca.

## Seguridad

⚠️ §826: esta sección no existía, y la regla 3 del PROCESO la pide. La clave de gasto no viaja, y este
RFC no la toca: cambia comprobaciones de la capa, el rango de los AIR de la capa, la atadura de la
identidad pública de la auditoría y la lectura del saldo en el cable. Lo que cierra lo dicen
«Motivación» y «Lo medido»: una resta envuelta que acuñaba valor en el envío, el cobro y la quema, y
una auditoría que atribuía su banda a otra cuenta.

## Lo que NO hace

- No sube el cable ni toca el OpenRPC. No mueve un KAT ni una cabeza firmada. ⚠️ §826: no subió el
  cable, y su E2 lo rompía: ver «Compatibilidad».
- No añade ni regenera un vector de `spec/vectors/`: lo que entra son tres tests negativos en los
  crates (el `public_id` ajeno en `stark-experiment`, la cota de banda en `zk-ssl`, el saldo no
  canónico en `zk-ssl-wire`).
- No toca los AIR del kit (`zk-ssl-air`), ya protegidos por `comprobar_enunciado` (D-3), ni sus
  vectores portables (`edad`, `pendiente`, `pago`, `prenda`), que son capturas no regenerables en
  `--sello`.
  ⚠️ §723: «ya protegidos» es falso para la banda y el cobro pendiente: ver la corrección de la D-3.
- No corre `--bancos` ni `--completo`: sólo `--sello`.

## Lo que corrige de RFC-0016

El §640 (RFC-0016) cerró con «ningún veredicto de un tercero depende» del lector de `u64` del cable
sin la regla canónica. E4 lo cierra para el saldo del cliente: el estado que el titular aporta se lee
canónico antes de que la capa lo use, de modo que un saldo escrito como `b + p` ya no descuadra la
conservación del suministro al reabrir. Queda nombrado en el §641.
⚠️ §778: este RFC decía «RFC-0015» en la fila E4, en el punto 4 de «Lo que hace», en el título de esta
sección y aquí, y «§631» como asiento de la frase citada: eran los números que la canonicidad llevaba
en su rama antes de renombrarse, que el §641 da como «hoy §631» y «ya renombrada de RFC-0015 a
RFC-0016». En el árbol, la regla del `u64` canónico es la E1 del RFC-0016, cuyo asiento es el §640,
y la frase citada está en el «Lo que NO cierra» de ese asiento; el RFC-0015 es el ciclo de vida del
firmante, y el §631, la E2a del RFC-0013. El §641 trae las mismas dos atribuciones y, como asiento,
no se reescribe.
