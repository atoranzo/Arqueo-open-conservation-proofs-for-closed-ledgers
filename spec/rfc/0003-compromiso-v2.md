# RFC-0003 — Compromiso de pendiente v2: Δ por pago e identidad de reembolso comprometidas

- **Estado:** PROPUESTO
- **Autores:** Che (con Claude, sesiones 56-57)
- **Fecha:** 2026-08-22 (r2; la r1 del 2026-08-21, `9bc8f30d3a14e0de`/182, se conserva para diff)
- **Versión del protocolo afectada:** zkssl/0.2 → zkssl/0.3 (rotura de formato)
- **Asiento(s) de AUDITORIA:** §178 (diseño de la caducidad), §180 (ejecución), §340 (el reloj atado), §342 (quién/cuándo son metadatos, medido) — y el **§343** (la adopción de este RFC)
- **Número: 0003, FIRME.** La regla 1 de `PROCESO.md` es numeración correlativa
  desde 0001, y el `0001` no falta: está **reservado** al endurecimiento del KDF
  del keystore (SHA-256 → Argon2id), «no está redactado» — nota de numeración del
  `0002` (`:30-34`) y `SECURITY.md:425-432` («no es un descuido, es una deuda con
  expediente»).

## Motivación

La entrada 12 del BACKLOG quedó abierta por la divergencia (ii): la política
del §119 pide «Δ=∞ **pago a pago, elección del emisor**» y hoy `T` es global
con knob (`set_refund_ttl`, persistida). El §342 midió además la afirmación
fuerte: **hoy quién puede reembolsar y cuándo son los dos metadatos del
operador** — `pending_meta[pos] = (sender_index, born)` y `refund_ttl`, ninguno
entra en entrada pública alguna (`RefundPublicInputs = {commitment, amount}`,
`CreditClimbPublicInputs = {root_old, root_new, amount}`). Eso es el diseño
declarado del §178 («pruebe quien pruebe»), no un defecto oculto; pero para una
infraestructura de liquidación con **cumplimiento verificable**, un tercero
debe poder comprobar que un reembolso respetó el plazo que el emisor eligió y
acreditó a quien el emisor comprometió.

La causa raíz está medida: el compromiso de producción es
`pending_commitment(receiver_id, salt, amount)` (`pending.rs:70-81`) — **tres
campos** — y su nativo `native_refund_commitment` (`circuit_refund.rs:76-91`)
declara ser idéntico. El Δ no cabe sin romper el formato, y `canon.sh` exige
los vectores `zkssl-0.2.json` **idénticos**. Este RFC paga esa rotura con el
proceso de versión.

## Diseño

### El principio que ordena todo

> **El compromiso contiene exactamente las elecciones del emisor, y nada más.
> Los hechos del operador van firmados aparte.**

- Dominio del emisor (dentro de C₂): `{receiver_id, salt, amount, f, Δ}`
- Dominio del operador (fuera, atestiguado): `{born, now}` — el `seq` entra en
  `epoch_digest_v3`, que el nodo firma con XMSS (atado por
  `la_altura_entra_en_el_digest`, §340)

### El compromiso

```text
C1 = M( M(receiver_id, salt), [amount,0,0,0] )        // v1, INTACTO como prefijo
X  = M( f, d(Δ) )                                     // el sobre de reversión
C2 = M( C1, X )                                       // v2 = cuatro merges de Rescue
```

donde `M` = merge de Rescue, `f = derive_public_id(refund_key)` (clave de
retorno del emisor, puede ser dedicada) y `Δ` = plazo en alturas de época,
elegido por el emisor. Es la forma exacta del prototipo
`t3a`/`t3b` de `pending.rs` («v2 = merge(v1, merge(refund_id, ·)): v1 como
prefijo», `:365`), **con la semántica del cuarto campo decidida** (ver D1).

### D1 — Δ RELATIVO, no expiry absoluto

`expiry = born + Δ` lo computa la capa; el emisor compromete **sólo Δ**.
Razones, por la vara del proyecto:

1. **Pureza**: `born` no es del emisor — lo fija la capa al aplicar. Un
   absoluto comprometido mezclaría una elección con una apuesta sobre un dato
   ajeno.
2. **El poder del §119.5**: el operador ordena la carrera de inclusión. Con
   absoluto, retrasar el `send` come la ventana del receptor; con Δ, la
   ventana es invariante ante retrasos.
3. **Δ=∞**: el centinela `Δ = u64::MAX` hace `now ≥ born+Δ` inalcanzable
   (aritmética saturante) — el «nadie nunca» del §119 es un caso particular,
   no una rama. ⚠️ Declarado: `u64::MAX` ya es centinela de OTRO campo
   (`REFUND_SENDER_NONE`, «sin emisor» ⇒ des-emisión, §180); no colisionan
   porque viven en campos distintos, y este RFC lo deja escrito.
4. Ninguno de los dos prototipos es exactamente esto: `t3a` compromete el
   `seq` de nacimiento y `t3b` un `expiry` absoluto (`ahora >= expiry`). Se
   adopta la FORMA del hash y se adapta la semántica — medido, no copiado.

### D2 — f DENTRO: el destino, comprometido y verificable sin revelarse

El circuito de reembolso v2 prueba: *conozco la apertura de C₂ **y** el
crédito va a la hoja de f* — con `f` como **testigo privado**. El molde
existe, **está en producción y está pagado**: es el TERCER MERGE del §117
(`circuit_claim.rs:883-896`) — «digest arrastrado y los CUATRO limbos del
rate := salt **testigo**», con su columna (`COL_LEAF_SALT`) transportada
constante. `C2 = M(C1, X)` es exactamente esa operación con otro testigo: D2
no propone un patrón, copia uno constreñido. (Y en `RefundAir`, además,
`get_assertions` deja las columnas 4..12 de la fila 0 libres como testigo,
`circuit_refund.rs:253-254`.) La subida de la cuenta destino ya viaja dentro
del cambio de raíz que `CreditClimbAir` ata. Un tercero verifica que el
acreditado fue el comprometido **sin saber quién es**.

**Relación con el §178, citado y superado**: el §178 eligió «Ni el compromiso
ni `claim` ni los discriminantes de §173 se tocan» y construir «entero sobre
piezas EXISTENTES» — es decir, no pagar la rotura de formato. Este RFC **es**
la rotura, pagada con proceso: la razón de no atar caduca con él. El ataque
que el §178 sí nombra (el aviso filtrado, robable en el diseño ingenuo) queda
cerrado por los MISMOS dos cerrojos, ahora con el primero verificable:
(1) el destino ya no lo fijan los registros sino el compromiso del emisor, y
(2) los materiales sólo los fabrica el emisor (salt de su hoja, §117) — el
prototipo lo ejercita («no eres el retorno», «un tercero pasó como retorno»).

**El principio «el emisor jamás en el árbol» se PRESERVA, y con una pieza
nueva**: el aviso al receptor lleva el sobre **X opaco**, no su apertura — el
receptor reconstruye `C2 = M(C1, X)` para su `claim` sin aprender ni `f` ni
`Δ`. ✅ **MEDIDO** (sesión 57, `PASTE-E1-M`), **y la expectativa de la r1
salió FALSA y se deja escrita**: la reconstrucción del claim NO toma `X` tal
cual — `C_PEND_IN` ata identidad (`COL_ACC_ID`, no `COL_R_ID`: §39) y
aleatorio (`COL_SALT`) con restricciones SEPARADAS (`circuit_claim.rs:996-1032`).
El claim v2 añade UNA fase de merge para el sobre: un ciclo (8 filas, con 208
libres tras `ROW_PENDING_ROOT`=815; caben 26), una bandera periódica, un
bloque de restricciones y 4 columnas de testigo (`TRACE_WIDTH` 55→59). Los
`CYC_*`/`ROW_*` se derivan por suma del anterior y **propagan solos**
(`:51-130`). El coste está derivado de las constantes, no supuesto.

### El invariante de reembolso, completo

```text
abre(C2) ∧ (now − born ≥ Δ) ∧ crédito → f
```

- la 1ª y la 3ª las prueba el circuito;
- la 2ª la verifica **cualquiera** con dos cabezas firmadas (la de `born` y la
  de `now`) y el `Δ` que la apertura del reembolso hace público — detectable
  con evidencia oponible, coherente con la vigilancia del §119.5.

### Qué se toca, medido

| pieza | cambio |
|---|---|
| `pending_commitment` + 5 sitios de producción (`two_phase.rs:343,433,640,1208`, `client.rs:347`) | pasan a componer/portar C₂ (o conviven, ver Compatibilidad) |
| `native_refund_commitment` (`circuit_refund.rs:76-91`) | gemelo v2 con `X` |
| `RefundAir` | `evaluate_transition` **NO cambia** (una vía de Rescue con flags, agnóstica al nº de merges — medido `:205-247`); cambian `build_trace`, `ROW_*` y `get_assertions` (traza ~×2: de 2 a 4 merges) |
| `ClaimAir` (`circuit_claim.rs`, `905495e70251de86`/2208) | **SÍ se toca, medido (sesión 57)** — asimetría con `RefundAir`: aquí el cosido es POR FASE, así que la transición SÍ gana un bloque (fase del sobre X), más un ciclo, una bandera periódica y `COL_X` (55..59). Molde: el tercer merge del §117 (`:883-896`) |
| `apply_send` / `apply_refund` / `apply_deissue` | el gate temporal pasa de `refund_ttl` global a `Δ` de la apertura; `refund_ttl` queda como TECHO sistémico o se retira (decisión del corte, declarada) |
| `spec/vectors/` | `zkssl-0.3.json` nuevo; `0.2` pasa al régimen del `0.1` (rechazado) |
| cable (`zk-ssl-wire`) | sólo si `Δ` se expone por RPC — hoy el cable tiene CERO líneas de caducidad (medido §341-M) |

## Compatibilidad

**Rompe el formato del compromiso ⇒ sube a `zkssl/0.3`.** Y la `0.3` está
**LIBRE**: la tabla de estado del RFC-0002 (`:13-19`) da sus TRES etapas
CERRADAS, con la 2 **redefinida sin romper el cable** (§210) — su línea `:24`,
que anunciaba «necesitará `zkssl/0.3`», caducó en ese mismo §210 y no se
actualizó (defecto del 0002, declarado y con corrección aparte).

| pieza | ¿rompe el cable? | vectores |
|---|---|---|
| compromiso v2 (C₂) | **sí** → `zkssl/0.3` | `zkssl-0.3.json` nuevos; **los de `0.2` y `0.1` SE CONSERVAN bajo su versión** — el fichero queda, y `conformance --check` los rechaza como «de OTRA versión»: el régimen ya practicado con el 0.1 (`RPC.md:9`) |
| circuitos (`RefundAirV2` · fase X del claim) | no viajan | sin efecto |
| cable (`zk-ssl-wire`) | **no** en E1-E3 — hoy tiene CERO líneas de caducidad (medido, §341-M) | sin efecto hasta E4, que traerá su propia sección |

**Regla 2 del PROCESO: los vectores viejos jamás se reescriben.**

El legado es inmune **por dominio, no por marca**: un C₁ no es apertura válida
de la forma C₂ (`t3b_v1_no_es_reversible` lo prueba contra 3 claves × 3
expiry) y los pendientes v1 sin meta ya son irrecuperables por el centinela
(§180). Convivencia hoja a hoja: cada pendiente lleva su versión implícita en
su forma; `claim` v1 sigue funcionando sobre hojas v1.

## Seguridad

**Efecto sobre el principio del API —la clave de gasto no viaja jamás:
NINGUNO.** El refund exige fabricar materiales, no exhibir claves; `f`, `Δ` y
las aperturas se generan y se quedan en el cliente.

- **§173 intacto**: C₂ es un `Digest` opaco; `f` y `Δ` sólo se abren al
  reembolsar, y ante el receptor ni siquiera entonces (X opaco).
- **§119.5 se REDUCE**: el operador ya no puede mover el plazo (vive en el
  compromiso) ni el destino (ídem); conserva ordenar la carrera y acelerar el
  reloj, ambos con evidencia oponible (cabezas firmadas).
- **B18.3 sin cambio**: el emisor con clave de retorno perdida no recupera —
  misma exposición declarada que hoy; `f` puede derivarse de una clave
  dedicada para aislar el riesgo.

**Efecto sobre las deudas declaradas:**

- **Aviso fuera de banda (§21): es la deuda que este RFC TOCA, y se declara.**
  El sobre `X` viaja EN el aviso, que sigue siendo fuera de banda. Un aviso
  filtrado queda donde el §178 lo dejó, con el primer cerrojo mejorado: `X` es
  opaco (el receptor no aprende `f` ni `Δ`) y los materiales sólo los fabrica
  el emisor (salt §117). La deuda no se agrava ni se salda: **cambia su carga**
  — ahora transporta también la reversión.
- **Nodo único: sin cambio.** Este RFC no introduce consenso en ninguna etapa,
  deliberadamente — `SECURITY.md` §6. La 2ª punta del invariante se verifica
  con cabezas **firmadas**, no con más nodos.
- **`--dev`: sin cambio.**

## Lo que se DESCARTÓ al medir

- **Expiry absoluto**: ventana comestible por el operador retrasando la
  inclusión (§119.5).
- **Δ en `pending_meta`**: igual de movible que hoy — no sería «elección del
  emisor», que es lo que la entrada 12 exige.
- **f sólo en registros**: no verificable por un tercero — es exactamente el
  estado actual que el §342 dejó medido.

## Etapas, con sus puertas

1. **E1 — vectores y nativos**: `commitment_v2` de producción + vectores 0.3 +
   conformance. Puerta: el conformance acepta 0.3 y rechaza 0.2 y 0.1.
2. **E2 — los circuitos**: `RefundAirV2` (traza, ROW_*, aserciones,
   crédito→f) **y la fase X del claim** (molde §117) — sesión propia, como el
   §178 manda («trabajo que estrena restricciones»). Puerta: el guardián de
   layout sube en uno por circuito que estrene restricciones, y el
   ladrón-con-aviso y el tercero-como-retorno en rojo.
3. **E3 — la capa**: `apply_send`/`apply_refund`/`apply_deissue` v2, el aviso
   con X, persistencia. Puerta: canon completo + la batería del §178
   (persistencia tras reinicio incluida).
4. **E4 (opcional) — el cable**: exponer Δ por RPC, con su versión de cable.

> ⚠️ **La puerta de E1, re-escalonada (§345).** Tal como quedó escrita
> arriba, «el conformance acepta 0.3» exige que el `run_send`/`run_claim` del
> escenario canónico PRODUZCAN compromisos v2 — el aviso con X y el claim que
> recompone C2 son materia de E3, y el escenario corre **pruebas reales**,
> así que también pisa E2. Medido en la sesión 57 sobre `conformance.rs`
> (`--emit`/`--check`, `fn escenario`). **E1 queda en los compositores de
> producción y sus tests** (`pending_commitment_v2`, `refund_envelope`, el
> gemelo nativo); la emisión 0.3 y el triple gate del conformance **se mudan
> a E3**. Mientras tanto `zkssl-0.2` sigue IDENTICO en cada canon: la
> convivencia, probada por el gate existente en vez de prometida.

> ⚠️ **La E2 tenia un hueco, medido y pagado (§352, 2026-08-23).**
> Las etapas nombraron `RefundAirV2` y la fase X del claim, pero el
> escenario del conformance tambien PRODUCE envios: el compromiso que
> el claim cobra lo deposita `circuit_send`, y ninguna etapa
> contemplaba el v2 del ENVIO. El §352 lo estrena como **E3c-1a** --
> `SendV2Air` (`circuit_send_v2.rs`): el cuarto merge `C2 = M(C1, X)`
> con el sobre LEIDO en `COL_X` (el patron del salt de hoja, §117),
> las entradas publicas del v1, y el sobre jamas publicado. La capa
> lo adopta en E3c-1b (los materiales ganan el sobre, dispatch en
> `prove_send`); la emision 0.3 y el giro del canon quedan en E3c-2.
>
> **Nota (2026-08-23, §353).** E3c-1b PAGADA: la via viva aprende el sobre
> (D-6) y la capa gana la guarda del ancho (D-7) -- el rechazo de una traza
> ajena es un `Err` de la capa, no un panico del Air. Queda E3c-2 (el
> escenario, la emision 0.3 y el giro del canon).

**El primer paso medible ya está dado** (sesión 57, `PASTE-E1-M`, salida
`b44b24915a67c268`/1175): `circuit_claim` abierto y medido. La reconstrucción
NO admite X tal cual; el claim v2 gana una fase de merge con coste DERIVADO
(8 filas de 208 libres) y molde propio (el tercer merge del §117). **Este
diseño ya no tiene piezas razonadas sin medir.**

## Referencias

AUDITORIA §178, §180, §340, §342 · BACKLOG entrada 12 (la (ii)) y nota 106 ·
`doc/CADUCIDAD_PENDIENTE.md` (`dbd7f95593d36180`/150) ·
`pending.rs` `t3a_reversion_como_segundo_cobro` (:353-452) y
`t3b_reversion_temporal_nativa` (:456-503) · `circuit_refund.rs`
(`5f4f04c51ee31a26`/469) · `circuit_claim.rs` (`905495e70251de86`/2208) ·
`PASTE-RFC-M` (salida `58af1bc0b1cbfe9f`/483) · `PASTE-E1-M` (salida
`b44b24915a67c268`/1175) · `PASTE-RFC-N` (salida `845c692ce353265b`/384) —
los tres sobre `6a1c1bb`.
