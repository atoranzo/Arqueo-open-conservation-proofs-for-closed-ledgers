# RFC-0007 — Las pruebas sobre el estado comprometido: el rechazo, la edad y los parámetros

- **Estado:** PROPUESTO — adoptado en el §450. Se conserva como registro de lo decidido, lo
  medido y lo descartado; cada etapa se sella con su puerta y su asiento.
- **Autores:** Che, con Claude (sesión 119)
- **Fecha:** 2026-09-09
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). La cabeza
  v5 y el `data` del error son aditivos en el cable; la versión de FORMATO viaja en la firma
  (4 → 5), como pasó de 2 a 3 (§292) y de 3 a 4 (§415).
- **Asiento(s) de AUDITORIA:** §178, §211, §246, §253, §275, §292, §321, §379, §387, §388,
  §404, §406, §413, §414, §415, §441; el §450, que lo adopta.
- **Hito:** H4 de la propuesta enviada a NLnet Restack (140 h), en sus palabras: *«Rejection
  with proof and the ageing proof. Circuits over the committed state: proof of the cause of a
  refusal; age distribution of what is in flight with its empty-box form and its concentration
  and cap variants; parameter consistency committed in the epoch head. Format and vectors.»*

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la cabeza v5 | `epoch_digest_v5`: UNA familia nueva bajo la firma —`params_digest` (los siete parámetros), `pmeta_root`, `next_pending`, `next_index`, `total_supply`—; `VersionCabeza` gana V5; el cable sirve los cinco campos y `zkssl_params` los tres parámetros que hoy no sirve; el testigo recompone y custodia; el mando acepta; KAT y fila en `NUCLEO.md`; vectores bajo su versión; testigos: recomponer rechaza una v5 sin uno de los cinco, y un parámetro cambiado en reposo cambia la cabeza | NO en el cable (claves aditivas); SÍ en la firma (formato 4 → 5) | **sellada** — §451 y §451-B (E1a: el núcleo y el mando aceptan v5), §452 y §452-B (E1b: `VERSION_FORMATO` 5, el nodo firma y sirve v5, el cable la exige por versión y el testigo la custodia) y §453 (el positivo v5 del catálogo del cable: una cabeza real) |
| E2 — el rechazo con causa, en el cable | el objeto de error gana `data`: la causa por su NOMBRE (la variante de `LayerError`), sus campos, y el `seq` de la cabeza en cuyo estado se juzgó; `message` no cambia; el catálogo de las veinticinco causas se publica en `RPC.md` y un test lo ata al enum (dos listas son dos productores) | NO (aditivo: ningún vector ni el OpenRPC pina el objeto de error) | **sellada** — §454 (`LayerError::causa` en la capa, `data` en el `-32000` y en la negativa de `zkssl_publishConsumo`, el catálogo en `spec/RPC.md` atado por test) |
| E3 — el rechazo con prueba, por caminos | una forma nueva del paquete de evidencia, `tipo: "rechazo"`, que el mando verifica sin nodo: la cabeza v5 firmada, la causa, y el material que la demuestra con caminos y aritmética pública sobre lo que la cabeza compromete; una tabla causa → material → qué revela; un positivo y un negativo por regla, derivados por mutación; su manifiesto y su puerta en el canon | NO (el paquete no cruza el cable) | **sellada** — §455 (E3a-1: `OverRegulatoryLimit`, `AccountLimitReached`, `ConsumoRepetido`, `ConsumoColision`), §456 (E3a-2: `StaleState`, `WrongRegulatoryLimit`, `DuplicateAccountInBatch`, `DuplicatePendingInBatch`, con un recibo real capturado por el proxy de un banco), §458-§459 (E3b: `AccountFrozen`, con el camino de congelados que el cable sirve al titular) y §460 (`SupplyCapExceeded`, con la petición del solicitante dentro del sobre). `AccountNotFound` pasa a E5 (CORRECCIÓN del §459); `PendingTreeExhausted` queda declarada sin prueba portable (CORRECCIÓN del §460) |
| E4 — la prueba de edad, medida primero | E4a: el coste en función de `next_pending`, con el instrumento antes que el circuito, y una puerta que decide si se construye o se declara un techo. E4b: el circuito sobre el RANGO `0..next_pending` de los árboles de pendientes y de meta, con la caja vacía, el tope y la concentración por emisor nombrado como formas de un mismo enunciado; su sobre, su manifiesto y sus vectores | NO | **en curso** — E4a sellada: el instrumento (§461, §461-B) y su medida y veredicto (§462): E4b se construye, con el techo declarado (CORRECCIÓN del §462); E4b abierta |
| E5 — las causas por circuito, y el kit verifica | `InsufficientBalance` con prueba de banda sobre la hoja comprometida (molde `circuit_audit`, sin el ciclo de titularidad); la re-verificación del STARK del solicitante para `ProofFailed` y `VerificationFailed`; un crate de AIR sólo-verificador que el mando consume, con `winter-verifier` en su clausura y el probador fuera | NO | abierta — depende de D-F |

Todas las medidas de este documento se tomaron sobre `bb02c71` (§449), en dos lecturas puras que
no escribieron un byte en el árbol: `PASTE-H4-M` y `PASTE-H4-M2` (ver Referencias).

### La frontera con H5b, y qué es de cada uno

H5b promete la completitud: *todo acuse comprometido resuelve en una época acotada como transición
aplicada o rechazo con prueba*. Este RFC entrega el **rechazo con prueba** (E2, E3, E5) y la
**caja vacía como propiedad de la distribución de edades** (E4: «nada más viejo que T sigue en
vuelo»). Lo que NO entrega, y queda para H5b, es la afirmación sobre los acuses: que cada uno
acabe aplicado o rechazado. La caja vacía habla de lo que está en el árbol de pendientes; la
completitud habla de lo que entró por el cable. Son dos objetos y dos pruebas, y mezclarlas
sería la clase de «dos frentes» que la ley prohíbe.

## Motivación

**Un rechazo es hoy prosa.** El nodo rechaza con `-32000` y un `message` que es el `Debug` de
`LayerError` (`crates/zk-ssl-node/src/main.rs`, el objeto de error se escribe a mano y no lleva
`data`). Un cliente rechazado sabe QUÉ regla invocó el operador porque lo lee; no puede
demostrar ante un tercero que esa regla se aplicaba de verdad al estado que la cabeza
compromete. Bajo la ley de esta casa el operador es otro adversario, y un rechazo que no se
puede verificar es una censura indistinguible de una regla. La tabla de propiedades de la
portada (`README.md`, `README_EN.md`, `doc/USE_CASES.md`) lo publica como *planeado* en su
fila 7, planeada (§444, §445).

**Las reglas se sirven sin firma.** `zkssl_params` devuelve `regulatoryLimit`, `maxSupply`,
`maxAccounts` y `custodianRoot`; en reposo viven siete parámetros (`meta:limit`,
`meta:max_supply`, `meta:max_accounts`, `meta:custodians`, `meta:governance`,
`meta:refund_ttl`, `meta:cust_max`, `crates/zk-ssl/src/persistence.rs`) y ninguno va bajo la
firma de la cabeza. `ARQUITECTURA.md` declara los parámetros inmutables y nada lo comprueba
desde fuera; el tope de custodios (`meta:cust_max`) se puede subir en reposo sin que nadie lo
cace (punto 48 de la cola de la 5.A). Toda prueba de rechazo cita una regla —una `T`, un
límite, un tope—, y una regla que no está comprometida no sostiene una prueba.

**La edad de lo en vuelo existe sólo en el juez del operador.** Cada pendiente tiene un
nacimiento (`born`, la altura al crear) en el árbol de meta —hoja `H(sender, born)`,
§388— y una `T` sistémica (§178, `DEFAULT_REFUND_TTL`) o un plazo por pago (`delta`, RFC-0003).
`RefundTooEarly { born, now, ttl }` es el único juez de edad, y es interno. La raíz de meta
(`root:pmeta`) se guarda y se comprueba al abrir desde el §388, pero **no está en la cabeza**
(punto 43): un tercero no puede hoy afirmar nada sobre cuánto tiempo lleva parado el dinero en
tránsito. `doc/USE_CASES.md` ya nombra «the "empty box" proof of what is in flight» como la
pieza del cierre de periodo; `doc/CADUCIDAD_PENDIENTE.md` diseñó el reloj del pendiente. Lo que
falta es la prueba.

**Y las tres piezas comparten la misma precondición.** Ninguna es demostrable sin que la cabeza
comprometa lo que hoy no compromete: los parámetros, la raíz de meta y los contadores que
acotan el universo. Por eso el RFC empieza por el formato (E1) y no por un circuito, y por eso
entra por RFC: toca `spec/RPC.md`, el OpenRPC y los vectores.

## Diseño

### D-A — El nombre: pruebas sobre el estado comprometido

Se llama así a toda prueba cuyo enunciado se verifica contra los campos que una cabeza firmada
compromete, sin el nodo y sin la capa: un camino, una aritmética pública o un STARK cuyas
entradas públicas son raíces y contadores de la cabeza. El paquete de evidencia (`RFC-0004`,
`spec/PAQUETE.md`) es el vehículo, y el mando `zk-ssl-verify` el consumidor. Las cinco formas
de hoy (posición v1, v2 con cofirmas, extensión, consumo, conflicto) son pruebas sobre el
estado comprometido; este RFC añade el **rechazo** (E3) y la **edad** (E4).

### D-B — La cabeza v5 lleva UNA familia: el estado que las pruebas necesitan

El RFC-0006 fijó en su D-3 que una versión de cabeza lleva una sola familia. Aquí la familia
es «lo que los rechazos y la edad necesitan y hoy no está firmado», y son cinco piezas, cada
una con la causa que sin ella no tiene prueba:

| pieza | qué es | sin ella no se prueba |
|---|---|---|
| `params_digest` | un solo digest de los siete parámetros, con dominio propio | ninguna causa que cite una regla: `OverRegulatoryLimit`, `WrongRegulatoryLimit`, `RefundTooEarly` (la `T`), `SupplyCapExceeded` (el tope), `AccountLimitReached`, `CustodianSetExhausted` (el cupo); y la consistencia de parámetros entre cabezas |
| `pmeta_root` | la raíz del árbol de meta de pendientes (`root:pmeta`, §388) | la edad de nada: `born` vive ahí |
| `next_pending` | la marca de agua del árbol de pendientes | el universo de la prueba de edad y `PendingTreeExhausted` |
| `next_index` | la marca de agua del árbol de cuentas | `AccountNotFound` (por índice) y `AccountLimitReached` |
| `total_supply` | el suministro (`meta:supply`) | `SupplyCapExceeded`, y la conservación del dinero vista desde fuera (el escalón que el punto 43 dejó abierto) |

**CORRECCIÓN (§247, escrita por el §452).** La fila de `next_index` lo llama «la marca de agua
del árbol de cuentas», y no lo es: desde F3 una cuenta se coloca por su identidad con sondeo
lineal (`crates/zk-ssl/src/accounts.rs`, `open_with_id`), y `next_index` queda como CENSO —la
cuota de altas contra `max_accounts`—, no como posición. Firmarlo sigue pagando
`AccountLimitReached`; lo que NO paga es `AccountNotFound(i)` por `i >= next_index`: la fila de
D-D que lo propone no demuestra nada, y E3 necesitará la AUSENCIA de `i` bajo `accountsRoot`.
El nombre no cambia: es el de la capa, el de `meta:next_index` y el del KAT del §451.

La composición sigue el molde de v3 y v4 —la envoltura de la anterior, sin tag de dominio,
porque el byte de versión del preámbulo es lo que separa las composiciones (§236)—:

```text
  v5 = merge( epoch_digest_v4(los once),
              merge( merge(params_digest, pmeta_root),
                     merge( as_digest(next_pending),
                            merge(as_digest(next_index), as_digest(total_supply)) ) ) )

  params_digest = merge( as_digest(DOMINIO_PARAMS),
                         merge( merge(as_digest(regulatory_limit), as_digest(max_supply)),
                                merge( merge(as_digest(max_accounts), custodian_set_root),
                                       merge( governance_set_root,
                                              merge(as_digest(refund_ttl),
                                                    as_digest(max_custodian_uses)) ) ) ) )
```

`DOMINIO_PARAMS` es un `u64` de ocho bytes ASCII como `DOMINIO_META_PENDIENTE` (`PMETA_V1`) y
sus hermanos, registrado en `zk-ssl-hash` y en su tabla de dominios. La forma exacta la fija
E1 con su KAT: si al medir sale una agrupación mejor, E1 la declara y este párrafo gana su
corrección; lo que no cambia es el conjunto de las cinco piezas ni que vayan bajo la firma.

**Génesis, declarado.** La primera cabeza v5 compone con los parámetros con que se abrió el
libro, la raíz del árbol de meta vacío, `next_pending = 0`, `next_index` igual al número de
cuentas abiertas al emitirla y `total_supply` igual al suministro en ese momento. No hay valor
especial: v5 compone lo que el libro tiene en esa cabeza, y las raíces vacías son las del árbol
vacío, como `cons_root` en v4.

**Qué compra.** (1) La *consistencia de parámetros* del hito: con dos cabezas firmadas del mismo
operador, `params_digest` iguales dicen que las reglas no se movieron; distintos, que se
movieron, y el único cambio legítimo —el conjunto de custodios por gobernanza,
`apply_governance_delegated`— deja un asiento `OpKind::Governance` en el registro. El testigo
lo anota como una clase de su diario; el mando lo comprueba con dos cabezas, como el sobre de
conflicto comprueba dos raíces. (2) El universo cerrado de la edad: `allocate_pending`
recorre `0..next_pending` buscando hueco y sólo sube la marca cuando no lo hay
(`crates/zk-ssl/src/two_phase.rs`, corregido en §211), así que **toda posición viva es menor
que `next_pending`**; firmarlo es firmar el universo. (3) El escalón 2 del punto 43: el
suministro bajo la firma, junto a las raíces que lo reparten.

**Qué cuesta, y dónde se paga.** `firmada()` del cable exige hoy la pareja de consumos con un
literal `== 4` declarado (punto 91): v5 exige las cinco piezas con el mismo molde, y el literal
crece a un `match` por versión. `linea_de_diario` del testigo custodia veintiún nombres por
lista escrita a mano (punto 89): gana cinco, y el test que la ata a `VistaFirmada` se mueve
con ella. `ParamsDto` gana `governanceRoot`, `refundTtl` y `maxCustodianUses`, aditivos. Todo
consumidor tipado de esta casa que lleve `deny_unknown_fields` deja de deserializar en el mismo
commit que lo causa, que es el precio ya declarado del RFC-0005 (D-B) y pagado en v4.

### D-C — La causa viaja como dato, no como prosa

El objeto de error de JSON-RPC admite `data`. Desde E2 un rechazo de la capa lleva:

```text
  "error": { "code": -32000,
             "message": "<sin cambio: el Debug de LayerError, con receptionSeq si lo hay>",
             "data": { "causa": "RefundTooEarly",
                       "campos": { "born": "0x..", "now": "0x..", "ttl": "0x.." },
                       "seq": "0x.." } }
```

`causa` es el nombre de la variante, tal cual; `campos` los suyos, con la codificación del
cable (QUANTITY en hex, digests como `Digest`); `seq` la altura de la cabeza en cuyo estado se
juzgó, que es lo que ata el rechazo a una cabeza que el cliente puede pedir firmada. `message`
no cambia: el testigo y los clientes lo leen y no se rompen. El catálogo —las veinticinco
causas con sus campos— se publica en `spec/RPC.md` y **un test lo ata al enum**: dos listas son
dos productores del mismo contrato, y aquí las ata el compilador (una variante nueva sin fila
se pone roja). `receptionSeq` sigue donde está y sigue sin ser evidencia oponible (§253): la
causa lo es sólo cuando E3 le pone la prueba.

**CORRECCIÓN (§247, escrita por el §454).** La medición del árbol precisa tres cosas de este
diseño. El ejemplo, `RefundTooEarly`, es una causa que el cable no sirve: sólo la producen
`apply_refund` y `apply_deissue`, y el nodo no despacha ninguno de los dos (qué causas emite hoy
el cable lo mide el asiento §454). Cinco variantes son TUPLA y no tienen nombre de campo en el
código: lo pone el catálogo (`index`, `detalle`), y tres llevan texto, no QUANTITY ni `Digest`.
Y `ConsumoRepetido` y `ConsumoColision` no viajan en el objeto de error sino en el `result` de
`zkssl_publishConsumo` (`accepted: false`): E2 les pone el mismo `data`.

### D-D — El sobre de rechazo, por caminos primero

E3 añade al paquete de evidencia la forma `tipo: "rechazo"`: la cabeza v5 firmada (campos,
digest, firma, `index`), la causa como en D-C, y el material que la demuestra. El mando
verifica la firma, recompone la cabeza, y juzga la causa contra lo que la cabeza compromete.
La tabla fija qué causas entran por caminos, qué material llevan y **qué revelan**, porque
revelar es un coste y se declara:

| causa | material | qué revela | etapa |
|---|---|---|---|
| `AccountFrozen(i)` | camino de `i` bajo `frozenRoot` (presencia) | que `i` está congelada | E3 |
| `AlreadyInThatFreezeState` | presencia o ausencia de `i` bajo `frozenRoot` | ídem | E3 |
| `ConsumoRepetido`, `ConsumoColision` | presencia bajo `consRoot` en la posición del consumo, con el ocupante | el consumo, que ya es público | E3 (reutiliza el sobre de consumo) |
| `RefundTooEarly` | la hoja de meta `(sender, born)` bajo `pmetaRoot` en la posición; `now` = `seq`; `ttl` = `refund_ttl` de `params_digest` (v1) o `delta` de la apertura (v2) | emisor y nacimiento; en v2, la apertura `(c1, f, delta)` | E3 por caminos; E5 la esconde |
| `RefundUnavailable` | ausencia de meta en la posición, o el centinela | la posición | E3 |
| `PendingMismatch` | la hoja bajo `pendingRoot` en la posición, frente al compromiso de los materiales | un digest | E3 |
| `StaleState` | la cabeza: la raíz declarada por el cliente no es la comprometida | nada | E3 |
| `WrongRegulatoryLimit`, `OverRegulatoryLimit` | aritmética pública con el límite de `params_digest` | el importe pedido, que el propio cliente pidió | E3 |
| `SupplyCapExceeded` | `total_supply` de la cabeza más el importe, frente a `max_supply` | el importe | E3 |
| `AccountLimitReached` | `next_index` de la cabeza frente a `max_accounts` | nada | E3 |
| `AccountNotFound(i)` | `i >= next_index` de la cabeza (las cuentas se abren densas desde cero; E3 lo mide antes de escribir el vector) | nada | E3 |
| `RecoveryToSameIdentity` | la identidad de la hoja de `i` bajo `accountsRoot`, frente a la pedida | el `public_id`, que es público | E3 |
| `DuplicateAccountInBatch`, `DuplicatePendingInBatch` | el propio lote | nada del estado | E3 |
| `PendingTreeExhausted` | `next_pending` igual a la capacidad, y que no hay hueco: la segunda mitad es la prueba de rango de E4 | nada | E3 la marca; E4 el hueco |
| `InsufficientBalance` | prueba de banda: saldo < pedido bajo `accountsRoot` sin revelar el saldo | nada más que la desigualdad | E5 |
| `ProofFailed`, `VerificationFailed` | el STARK y las entradas públicas que el solicitante envió, re-verificados | lo que el solicitante ya envió | E5 |
| `CustodianSetExhausted` | el cupo se deriva del registro (§393, §394), que la cabeza compromete sólo por `chainDigest` | — | declarada sin prueba portable |
| `NotTheIssuer`, `NotTheAccountHolder` | la autorización ausente no se puede exhibir | — | declaradas sin prueba |
| `BalanceOutsideBand` | es del camino de auditoría del titular, no de una operación | — | fuera |
| `Store` | fallo del operador, no una regla | — | se declara como fallo, nunca como rechazo |

Cada causa de E3 lleva un vector positivo producido por un banco con nodo real y un negativo
por mutación del material, con el texto de rechazo tomado del binario en la corrida (regla de
`spec/PAQUETE.md`, sección 9); el manifiesto pina código de salida y texto; el canon lo corre
como corre los otros catálogos. La cabeza que va dentro es v5: por eso E3 viene después de E1.

**CORRECCIÓN (§247, escrita por el §455).** La medición del árbol, al abrir E3, precisa cuatro
cosas de esta tabla. Cinco de sus causas no las sirve el cable —`RefundTooEarly`,
`RefundUnavailable`, `PendingMismatch`, `RecoveryToSameIdentity` y `AlreadyInThatFreezeState`— y
probarlas no le sirve a ningún cliente: salen de E3. La cabeza no es siempre la del `seq` exacto:
lo que sólo crece —`nextIndex`, los consumos, las cuentas— se prueba con una cabeza anterior o
posterior según la mutabilidad medida, y sólo `StaleState`, `AccountFrozen` y
`SupplyCapExceeded` exigen la exacta, que existe si nadie escribió hasta el siguiente latido. Los
caminos de congelados y de ausencia en cuentas NO los sirve hoy ningún método: E3b añade dos, y
eso toca el cable (aditivo: `zkssl/0.3` no se mueve). Y un camino revela a sus vecinos: el de
ausencia de `i` deja enumerar sus saldos y el de congelados su estado, así que «qué revela: nada»
no vale para `AccountNotFound`. E3 queda partida en E3a-1 (§455: las cuatro que se provocan sin
recibo), E3a-2 (las que exigen un recibo probado) y E3b.

**CORRECCIÓN (§247, escrita por el §459).** La del §455 dice dos cosas que la medición de la
sesión 125 corrige. Los caminos de congelados y de ausencia NO se sirven con dos métodos: el §458
sirve uno, `zkssl_frozenPath`, con la credencial del titular (§261) —el estado de congelación es
suyo: `two_phase.rs` lo comprueba después de la autoridad para no filtrarlo—, y un grifo del
sandbox, `dev_freeze`; la superficie pasa de 26 a 28 métodos y `zkssl/0.3` no se mueve. Y un
camino de ausencia de `i` NO deja enumerar saldos: las hojas de cuenta v7 llevan sal (§117). Lo
que revela es OCUPACIÓN —cada hermano igual al vacío de su nivel dice que ese subárbol no tiene
cuentas—, y con `zkssl_publicId`, que no pide credencial, eso reabriría la enumeración de
identidades que F3 cerró. Por eso `AccountNotFound` sale de E3 y va a E5, como prueba de
conocimiento cero de hoja vacía (decisión del autor, sesión 125). `AccountFrozen` se prueba desde
el §459 con la hoja bajo el `frozenRoot` de la cabeza del `seq` exacto; la profundidad la fija el
núcleo (`FROZEN_DEPTH`, `spec/NUCLEO.md`), porque `native_merge` no separa hoja de nodo.

**CORRECCIÓN (§247, escrita por el §460).** La medición de la sesión 126 cierra las dos filas
que la tabla dejaba en E3. `SupplyCapExceeded` la producen dos vías de la capa
(`apply_mint_delegated` y `apply_mint_pending_delegated`), las dos antes de verificar prueba
alguna, y por el cable sólo la alcanza el grifo del sandbox, `dev_fund`: no hay método de
emisión de producción. Se prueba desde el §460 con la cabeza del `seq` exacto y un material que
la tabla no nombraba: la `peticion`, los `params` de la emisión rechazada tal cual los envió el
solicitante. Sin ella el `wouldBe` sería palabra del nodo, y un nodo que rechazara un importe
que cabe obtendría una prueba verde; con ella, `wouldBe` tiene que ser el `totalSupply` de la
cabeza más el importe pedido (decisión del autor, sesión 126). `PendingTreeExhausted` NO se
prueba: `allocate_pending` cuenta como ocupadas las posiciones RESERVADAS, y las reservas no se
persisten ni van bajo la firma —su duración es política del nodo—, así que `next_pending`
igual a la capacidad no es condición necesaria ni suficiente; sólo lo sería el árbol
comprometido lleno, 2^32 hojas que ningún banco produce. Queda declarada sin prueba portable, como
`CustodianSetExhausted` (decisión del autor, sesión 126). Con esto E3 queda sellada entera.

### D-E — La prueba de edad es sobre un RANGO, no sobre n caminos

El enunciado: sobre las posiciones `0..next_pending` de los árboles de pendientes y de meta
—cuyas raíces y cuya marca la cabeza v5 firma—, la distribución de edades de las posiciones
vivas, con `edad = seq - born`. La forma del circuito viene de la estructura, medida:

- **Universo cerrado**: toda posición viva es menor que `next_pending` (D-B, §211). Las
  posiciones vacías llevan la hoja cero en los dos árboles.
- **Reconstruir la raíz del rango cuesta lo que un árbol completo sobre `n` hojas**: del orden
  de `2·n` hashes por árbol, más la espina hasta la raíz con los hermanos de subárbol vacío
  como constantes públicas; no `n` caminos de treinta y dos. Por cada posición viva, abrir el
  compromiso del pendiente (`pending_commitment` y, en v2, el sobre de reversión) y la hoja de
  meta son unos pocos hashes más. Con el molde de `circuit_audit` —ocho filas por hash, una
  subida de treinta y dos niveles en 256 filas—, `n = 4096` da un orden de 2^17 a 2^18 filas.
  **Es una proyección, no una medida**, y por eso existe E4a.
- **Un enunciado, tres formas**, como `circuit_audit` cubre tres auditorías con dos cotas
  públicas: la **caja vacía** (`count(edad >= T) == 0`, con `T` pública: nada más viejo que `T`
  sigue en vuelo), el **tope** (`suma(importe) <= cap` o `count <= cap`, con la cota pública) y
  la **concentración** (para un emisor nombrado `s`, `suma(importe con sender == s) <= X`). La
  concentración global —el máximo sobre todos los emisores— exige agrupar dentro del circuito
  y es la forma con menos confianza: E4a mide su coste y E4b la construye sólo si cabe; si no, se
  declara y se sirve la forma por emisor nombrado.
- **Entradas públicas**: `pendingRoot`, `pmetaRoot`, `next_pending` y `seq` de la cabeza v5,
  más las cotas del enunciado. **Testigo** del operador: por cada posición viva, la apertura del
  compromiso (receptor, sal, importe; y `f`, `delta` en v2) y `(sender, born)`; el operador los
  tiene todos (los recibe en `sendMaterials` y los guarda en `pamt:` y `pmeta:`).
- **Lo que NO prueba**: nada sobre lo que nunca entró en el árbol (un pago que el operador no
  aplicó no tiene edad: eso es H5b), nada sobre saldos, y nada sobre pendientes de otra época
  que la de la cabeza que firma las raíces.

**E4a, la puerta.** Antes de escribir el AIR: el número de hashes en función de `n` medido
sobre las hojas reales de un libro del banco, la proyección de filas con el molde, y un tiempo
de prueba medido en la máquina de referencia para tres `n` crecientes con la configuración de
`proof_options`. La puerta decide: si la prueba para el `n` del libro de referencia cabe en un
tiempo declarado, E4b la construye; si no, este RFC declara el techo de `n` para el que la
prueba es practicable, y el hito lo dice con esa cifra. Un techo declarado con medida vale;
una prueba que no se puede producir, no.

**CORRECCIÓN (§247, escrita por el §462).** La E4a se midió en la sesión 126 y precisa cuatro
cosas de esta sección. **Una**: la proyección de arriba («`n = 4096` da un orden de 2^17 a 2^18
filas») contaba los hashes de los árboles y omitía la comparación de edad —una descomposición
en bits por posición viva— y el relleno a potencia de dos que exige el probador. **Dos**: las
«hojas reales de un libro del banco» no hacían falta: el número de hashes del rango depende
sólo de `n`, no de las hojas ni de los huecos, y el instrumento lo prueba con hojas de los
productores reales (`pending_commitment`, `meta_pendiente_hoja`) cruzadas contra el `SparseTree`
de la capa. **Tres**: la puerta tiene ya sus cifras, fijadas por los principios del proyecto
antes de medir (decisión delegada por el autor, asiento §461): el tiempo es un latido, 60 s
(`LATIDO_POR_DEFECTO_S`, §121), y la memoria, la mitad de la RAM física, medida en la corrida.

**Cuatro, lo medido.** El instrumento del §461 —la subida Merkle de la casa con la profundidad
como parámetro, que envuelve `MerkleAir` sin copiarle una restricción— prueba y verifica los
hashes de los dos árboles con `proof_options`, en la máquina de referencia (i5-1135G7, 8
núcleos, 12.248.696 kB de RAM, WSL2) y sobre el commit `808a093`:

| `n` | filas | probar | verificar | prueba | pico |
|---|---|---|---|---|---|
| 1024 | 32.768 | 2,43 s | 1,1 ms | 93.153 B | 250.344 kB |
| 4096 | 131.072 | 10,09 s | 1,3 ms | 111.660 B | 983.532 kB |
| 16384 | 524.288 | 44,94 s | 1,6 ms | 134.975 B | 3.915.228 kB |

El coste por fila es casi constante (74, 77 y 86 µs; unos 7,5 kB). El veredicto, con la regla de
arriba y el `n` de referencia que esta misma sección escribió antes de medir, 4096: los árboles
cuestan 10 s, y la prueba completa proyectada —las aperturas de cada hoja viva y la comparación,
a la misma escala por fila— rellena a 2^18 filas, del orden de 22 s. **Cabe: E4b se construye.**
El techo se declara ya, en dos cifras que no se mezclan: **medido**, los árboles caben en un
latido y en media RAM hasta `n` = 32.750 (la talla de 2^19 filas); **proyectado**, la prueba
completa, hasta `n` ≈ 8192. E4b mide su AIR real y sustituye la proyección por la medida; si la
prueba completa para 4096 no cupiera en un latido, el techo se declara entonces, con esa cifra.

### D-F — El kit verifica pruebas: el AIR entra en la clausura, el probador no

Hoy `zk-ssl-verify` depende de `serde_json`, `xmss`, `zk-ssl-hash` y, para los tests,
`winter-math`; ningún `winter-air` ni `winter-verifier` vive en el árbol fuera del paraguas
`winterfell`, que arrastra el probador (`crates/zk-ssl-verify/Cargo.toml`, `Cargo.toml`). Una
prueba de rechazo o de edad que sólo el operador pudiera verificar no sería una prueba: la
autoridad final es la prueba, y quien la juzga no puede ser quien la emite. Por eso E5 —y E4b
si su puerta abre— entregan un crate de AIR **sólo-verificador**: los AIR sin `build_trace` ni
`Prover`, sobre los cuatro sub-crates de winter sueltos y clavados, como ya se midió fuera del
árbol en el spike del PLAN-22 (`air-solo`: el verificador del envío corre en ~2,4 ms sin la
fachada).
`zk-ssl-verify` lo consume; `winter-verifier` entra en la clausura del kit y el probador sigue
fuera. La frase de `crates/zk-ssl-verify/Cargo.toml` que dice «ninguno con prover ni verifier
de winterfell» gana entonces su corrección §247, no se reescribe. Lo que H2 prometió —sin el
repositorio, sin el autor, sin red, sin telemetría— sigue en pie: el kit crece, no cambia de
naturaleza.

## Lo que se DESCARTÓ al medir

1. **Un hash del verificador vigente en la cabeza** (la entrada 83 del `BACKLOG.md`, F4). El
   AIR es código, no datos: lo único hasheable en ejecución son las `ProofOptions`, y un
   operador puede cambiar el AIR dejándolas idénticas (§246, §321, `crates/zk-ssl/src/log.rs`).
   Este RFC firma la mitad **datos** de esa cadena —los parámetros— y deja la mitad AIR
   bloqueada con su razón, que depende de la entrada 55 o de compilación reproducible (H6).
2. **`n` caminos para la edad.** Probar cada posición viva por su camino cuesta `n·32` hashes y
   no demuestra que no haya otras: la reconstrucción del rango cuesta `2·n` y sí lo demuestra.
3. **Un histograma mantenido por transición.** Que cada operación actualice un acumulador de
   edades firmado exigiría tocar todos los circuitos de transición y sus vectores; una prueba
   por época sobre el estado comprometido no toca ninguno.
4. **Verificar sólo en la referencia.** Descartado en D-F: una prueba que sólo verifica el
   operador no cambia la confianza residual.
5. **Cambiar `message`.** El testigo y los clientes lo leen; la causa entra en `data`, aditiva,
   y `message` sigue diciendo lo mismo.
6. **Una versión de cabeza por pieza.** Cinco versiones para cinco piezas que se necesitan a la
   vez serían cinco eras silenciosas; la ley pide menos excepciones. Una versión, una familia,
   y el RFC-0006 D-3 se respeta en su sentido: una familia por versión.
7. **`receptionSeq` como prueba del rechazo.** No es evidencia oponible (§253) y no cambia: la
   prueba es el material de E3, no el contador.

## Compatibilidad

- **El cable NO sube.** Las cinco claves de v5 en `zkssl_epochHead` y `zkssl_signedEpochHead`,
  las tres nuevas de `zkssl_params` y el `data` del error son aditivas; `zkssl/0.3` sigue, por
  la misma regla que no subió con `acusesRoot`, `mmrRoot` ni `consRoot`. La versión de FORMATO
  de la firma pasa de 4 a 5 y **la elige el recompositor**: `VersionCabeza` gana `V5`, y un
  consumidor que no la conozca rechaza con su texto (§404, §406).
- **Los vectores no se reescriben.** Los tres escenarios `zkssl-0.N.json` y los once vectores
  del cable siguen siendo cabezas v3; los del paquete llevan cabezas v2 y v3, y los del consumo y
  del conflicto, v4: todos siguen. E1 emite su positivo v5 y sus negativos bajo su versión; E3 y
  E4 nacen con catálogo propio.
- **`deny_unknown_fields`, el precio.** Los consumidores tipados de esta casa que lean
  `SignedEpochHeadDto`, `EpochHeadDto` o `ParamsDto` dejan de deserializar los campos nuevos
  hasta el commit que los añade, que es el mismo: la rotura ocurre dentro del sello, como en v4.
- **El `data` del error.** Ningún vector, ni el OpenRPC, ni el testigo pinan el objeto de error
  más allá de `code` y `message`: añadir `data` no rompe a nadie que exista hoy. El OpenRPC se
  regenera con su generador y el bloque gatea que el diff sea exactamente el esperado.
- **Un libro anterior a E1 abre**: v5 se compone de lo que ya está en reposo; no hay clave nueva
  en el almacén, así que no crece la deuda de migración del punto 52.

### Por qué entra por RFC

`spec/rfc/PROCESO.md` acota el RFC a lo que cruza el cable, y este cambio cruza el cable tres
veces: las claves nuevas de la cabeza y de los parámetros en `spec/RPC.md`, el `data` del
error, y los vectores nuevos. Y por la regla 4: ACEPTADO exige la spec al día, el OpenRPC
regenerado, los vectores bajo su versión y las suites verdes, etapa a etapa.

## Seguridad

- **La clave de gasto no viaja.** Ninguna etapa la toca: las pruebas de E3 las produce el
  operador con lo que ya tiene, y las de E5 con lo que ya recibió.
- **Un rechazo con prueba demuestra que la regla se aplicó, no que la regla sea justa.** Lo que
  compra es que la censura tenga que disfrazarse de regla y que ese disfraz sea verificable.
- **Los parámetros comprometidos hacen visible un cambio; no lo impiden.** Quien controla el
  nodo sigue pudiendo cambiar `meta:limit` en reposo; lo que cambia es que la siguiente cabeza
  lo dice, y que dos cabezas con `params_digest` distintos y sin asiento de gobernanza son
  evidencia.
- **La caja vacía no habla de lo que nunca entró.** Un pago que el operador no aplicó no está en
  el árbol y no tiene edad: la prueba de edad no lo ve. Eso es la completitud, y es H5b.
- **La completitud del universo descansa en la marca de agua.** Que toda posición viva sea menor
  que `next_pending` es hoy una propiedad del código (`allocate_pending`), no un invariante
  probado; E1 la firma y E4a le pone un testigo negativo: un libro con una posición viva por
  encima de la marca tiene que romper la prueba.
- **Un `PendingTreeExhausted` no es refutable.** Las reservas de posición no van bajo la firma:
  con el árbol no lleno, el operador puede alegar que se agotó y nadie puede probar lo
  contrario. Con 2^32 posiciones el alegato es inverosímil, no imposible, y se declara
  (corrección del §460).
- **Cada prueba de E3 revela lo que su fila dice.** `RefundTooEarly` por caminos revela emisor,
  nacimiento y, en v2, la apertura; quien no quiera revelarlo espera a E5. Nada revela un saldo.
- **Deudas declaradas que siguen**: aviso fuera de banda (§21), nodo único, `--dev`; y la
  concentración global, que puede no caber.

## Referencias

- El hito, verbatim, en la línea 44 del formulario enviado (`NLNET-form-answers-EN-v3.txt`,
  huella `26dcde32091e857d`; el bloque 44..45, `cc71d28bfbf28de3`).
- Las dos lecturas puras: `PASTE-H4-M` (`b97fa3671a204fd8`; salida `874081d54f78eab9`, 2191
  líneas) y `PASTE-H4-M2` (`ab3c84deb15111db`; salida `3609f88610ec6480`, 1444 líneas), sobre
  `bb02c71`.
- El código medido: `crates/zk-ssl/src/lib.rs` (el enum `LayerError`), `two_phase.rs`
  (`allocate_pending`, `RefundTooEarly`), `persistence.rs` (los siete parámetros y las raíces en
  reposo), `sparse_tree.rs` (profundidad 32), `crates/zk-ssl-hash/src/lib.rs`
  (`epoch_digest_v4`, `meta_pendiente_hoja`, los dominios), `crates/zk-ssl-wire/src/lib.rs`
  (`ParamsDto`, `VistaFirmada`, `firmada`), `crates/zk-ssl-cli/src/witness.rs`
  (`linea_de_diario`, `recomponer`), `crates/zk-ssl-node/src/main.rs` (el objeto de error, los
  brazos `zkssl_params` y `zkssl_supply`), `crates/stark-experiment/src/circuit_audit.rs` (el
  molde), `crates/zk-ssl-verify/Cargo.toml` (la clausura).
- Los asientos: §178 (la `T`), §211 (`allocate_pending`), §236 (el byte de versión), §246 y
  §321 (por qué no hay hash del verificador), §253 (`receptionSeq`), §275, §292, §414 y §415
  (las tres composiciones anteriores), §379 (la conservación), §387 y §388 (las raíces del
  pendiente y de su meta), §393 y §394 (el cupo), §404 y §406 (`VersionCabeza`), §413 (el
  consumo), §441 (el RFC-0006 aceptado).
- Los documentos: `spec/rfc/0004-paquete-de-evidencia.md`, `spec/rfc/0005-nucleo-congelado.md`
  (D-B: lo firmado crece sólo por versión), `spec/rfc/0006-consumo-publicado.md` (D-3: una
  familia por versión; el sobre de conflicto), `spec/PAQUETE.md`, `spec/NUCLEO.md`,
  `doc/CADUCIDAD_PENDIENTE.md`, `doc/USE_CASES.md`, `SECURITY.md`; en `BACKLOG.md`, las entradas
  54, 55 y 83; en la cola de la 5.A, los puntos 43, 48, 89 y 91.
