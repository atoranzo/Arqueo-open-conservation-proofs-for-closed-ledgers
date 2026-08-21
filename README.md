# ZK-SSL — capa de liquidación con privacidad y cumplimiento demostrable

Un sistema de liquidación donde las transferencias son privadas, el
cumplimiento normativo es **demostrable criptográficamente**, y **no hace
falta confiar en ninguna ceremonia de setup**.

Más el trabajo comparativo que fundamentó su diseño: **el mismo circuito
implementado en cinco paradigmas de prueba**, medido en condiciones
idénticas.

```rust
let layer = SovereignLayer::open("./ledger", custodios, gobernanza, limite, tope, max_cuentas)?;

// FASE 1 — el pagador envía. La capa no ve su clave.
let m = layer.send_materials(alice, id_de_bob, 250_000, aleatorio)?;
let envio = client::prove_send(&m, clave_alice, proof_options())?;  // LOCAL
layer.apply_send(&envio, alice, &estado_alice, 250_000)?;

// FASE 2 — el receptor cobra. Tampoco ve la suya.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, clave_bob, proof_options())?;   // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;
```

**Dos fases, y no por gusto.** Una liquidación que actualiza las dos cuentas
en una sola transición exige que quien construye la prueba **conozca los dos
saldos**: pagar a alguien revelaría cuánto tiene. Un envío toca una sola
hoja; del receptor basta su identificador público.

**Y ninguna clave llega a la capa.** Lo que la capa entrega son caminos y
raíces —datos públicos—; lo que recibe son pruebas que verifica. Lo demuestra
`a_whole_payment_without_giving_any_key_to_the_layer`.

⚠️ **Con dos costes declarados**: el pago no es firme hasta que se cobra, y
si el receptor nunca cobra **el importe queda inmovilizado hasta que el emisor
lo reembolse** (§178-§181). ⚠️ Ese plazo **no se cuenta en tiempo, sino en
entradas del registro**, y quien las hace avanzar es el operador. Los pendientes
anteriores a ese mecanismo no llevan destino anotado y **son irreembolsables**.
Ver `AUDITORIA.md` §29 y §30.

---

## Pruébalo en cinco minutos

Rust estable, `--release` obligatorio (ver más abajo por qué). Cuatro
comandos, cuatro propiedades:

```bash
# 1 · Un pago completo (fondear ×2 → enviar → cobrar) con pruebas STARK
#     reales y traza por fases — saldos finales 750000/1250000, cadena
#     de transiciones íntegra:
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# 2 · El nodo de referencia vivo + el SDK con claves ALEATORIAS — el
#     pago viaja por JSON-RPC y la clave de gasto NO (se prueba en local):
cargo run --release -p zk-ssl-node -- --dev &
cargo run --release -p zk-ssl-sdk --example e2e
kill %1

# 3 · ¿Vas a escribir una SEGUNDA implementación? Este es tu contrato:
#     re-ejecuta el escenario canónico y compara campo a campo:
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json

# 4 · El wallet en reposo, cifrado con la MISMA construcción que el
#     ledger y dominio propio (guardar → cargar → que lo malo FALLE):
cargo run --release -p zk-ssl-sdk --example keystore
```

Las tres líneas que deben salir: `cadena de transiciones íntegra`,
`E2E OK: … la clave de gasto no viajo`, `CONFORMIDAD: … todo IDENTICO`.

---

## De implementación a protocolo (§197–§199)

Desde agosto de 2026 esto no es solo una capa: es un **protocolo con
contrato público**, pensado para que exista una segunda implementación
sin leer el código del nodo.

| Pieza | Qué fija |
|---|---|
| [`spec/RPC.md`](./spec/RPC.md) | La especificación normativa (**`zkssl/0.2`** desde §209): 17 métodos JSON-RPC, tipos de cable en hex canónico — el mismo byte a byte que persiste la capa. Los vectores de `0.1` **se conservan** |
| [`spec/openrpc.json`](./spec/openrpc.json) | **GENERADO** desde la tabla que vive en `zk-ssl-wire`, junto a los DTOs: una sola fuente. El verificador de cada sello exige que regenerarlo lo reproduzca **byte a byte** |
| [`spec/vectors/`](./spec/vectors/) | Vectores de conformidad **por versión**: el escenario canónico reducido a hechos (raíces, digests de prueba, cadena, cabeza de época). El determinismo por operación está medido en tres cruces independientes y es **compuerta permanente** |
| [`spec/rfc/`](./spec/rfc/) | El proceso de cambio: numerado, con estados, y una regla de oro — los vectores viejos **jamás se reescriben**; si el cable cambia, la versión sube |
| `crates/zk-ssl-wire` | El formato de cable (DTOs validados, `deny_unknown_fields`) |
| `crates/zk-ssl-node` | El nodo de referencia (axum): entrega materiales, verifica pruebas; `dev_*` doblemente cerrado (feature + flag) |
| `crates/zk-ssl-sdk` | El lado del titular: la única línea donde interviene la clave de gasto es `prove_send/prove_claim`, **en local**. Incluye el **keystore**: wallet en reposo con la misma construcción que el ledger y dominio propio — un test EXIGE que la clave del ledger no abra el keystore |
| `crates/zk-ssl-cli` | Sandbox y trazador sobre la capa REAL (`simulate` · `trace-tx` · `inspect-state` · `conformance`), salida JSONL opcional |

Actado en ejecución, no solo en código: el pago SDK↔nodo con claves
aleatorias terminó con alice 750000 · bob 250000 · `verifyChain ok`, y
`spend_key` no tiene ni `Serialize` — no puede viajar por accidente.

---

## Formalización en marcha (`doc/fv/`)

Los circuitos del núcleo de pagos tienen **ESPEC ejecutable**: un
intérprete fino en Python reproduce byte a byte la salida patrón-oro
del circuito Rust — mutantes incluidos — y una compuerta cuenta cada
celda de la traza con dueño declarado (`circuit_send`: 23 clases ·
1288 celdas-clase · 0 sin dueño; `circuit_claim`: 21 · 1155 · 0).
`tools/check_constraint_layout.py` (el guardián) barre los **28
circuitos** y canta **seis censos** en línea limpia en cada sello.
Mapa y estado: [`doc/fv/mapa_fv_capas.md`](./doc/fv/mapa_fv_capas.md) y
[`doc/VERIFICACION_FORMAL.md`](./doc/VERIFICACION_FORMAL.md). El techo
de todo esto está declarado, no escondido: `AUDITORIA.md` §§69
(Winterfell asumido).

---

## ⚠️ Léelo antes que nada: el operador es un intermediario de confianza

Esto es un **nodo único**. Quien lo opera:

- **Ve todos los saldos.** No frente a quien mantiene el estado.
- **Ordena las operaciones y puede censurar.**

⚠️ **Y la privacidad frente a TERCEROS tampoco esta.** Este documento decia
que lo estaba «frente a terceros **que solo ven pruebas**». La condicion
—que un tercero solo vea pruebas— **es falsa**, y esta medido el 31-07-2026
(`AUDITORIA.md` §93). **Desde la entrada 50 (§156-§157) ambas
superficies cambiaron de estado** — la tabla conserva la medida y añade
el veredicto de hoy:

| superficie | medido → veredicto |
|---|---|
| `account_view`, `balance_of`, `nonce_of` y `public_id_of` | del OPERADOR por diseño (§129); el titular tiene `account_view_authenticated` (49-A). Y los índices **ya no son enumerables**: colocación `public_id mod capacidad` (F3, §157) — el contrato `account_indices_are_not_enumerable` vigila en verde |
| El camino Merkle que el protocolo entrega | `siblings[0]` sigue siendo la hoja del vecino, pero desde el flip D4 (§156) la hoja va **ENVUELTA** —`native_leaf_salted(…, salt)`, salt fijado al abrir (§117)—. El diccionario que recuperó el saldo en **10,84 s** ya **no acierta**: el instrumento quedó como CONTRATO |

El coste de aquel ataque **no era un número, era una curva**: **2,4 min**
para 0-10.000 EUR, 4,1 h para 0-1 M, y 8,3×10^7 años-núcleo en 64 bits
uniformes —que nunca lo son en dinero—. **La curva está muerta por el
salt**; se conserva como medida del riesgo que se cerró.

⚠️ Y el vecino de árbol **era elegible** —altas consecutivas—. Desde F3
la colocación es `public_id mod capacidad`: el contrato
`account_indices_are_not_predictable` vigila en verde que elegir vecino
ya no sea posible (§157).

**Alcance, acotado**: **1 cuenta** por camino —solo `siblings[0]` es
preimagen de hoja; los otros 31 hermanos son raices de subarbol y no son
diccionariables—.

**Estado**: entradas 49 y 50 del backlog, **abiertas**. La primera se cierra
facil y **no cierra la segunda**, que exige tocar el formato del compromiso
y decidir si el cliente custodia estado (§93.4).

Ambas cosas exigen consenso distribuido, que **no está implementado** y es
un problema de otra disciplina.

Lo que sí se cerró: **no puede reescribir el historial en secreto**
(registro encadenado de transiciones), ni crear dinero, ni gastar de una
cuenta ajena, ni operar sobre un estado corrupto.

⚠️ **La primera lleva condición, y conviene decirla.** Un registro encadenado
sólo impide reescrituras **detectables por quien ya vio una cabeza anterior**.
Lo que cambió es que **hoy esa condición se puede cumplir sin pedirle nada al
operador**: el nodo sirve la cabeza **firmada** —`zkssl_signedEpochHead`— y la
CLI trae un **testigo** de referencia que la verifica con `zk-ssl-verify` y
**fija la clave que ve la primera vez**. Desde ese primer encuentro, el
operador no puede cambiar de clave sin que un tercero lo vea.

**La garantía la tiene quien mira, no quien lee**: sin un testigo corriendo, la
frase de arriba no protege a nadie.

**Qué es esto**: una demostración de que las propiedades criptográficas de
una liquidación soberana son construibles y medibles.
**Qué no es**: una capa descentralizada.

---

## Números medidos

Todos en release, misma máquina. Una sola ejecución: sirven para comparar
órdenes de magnitud, **no como benchmark**.

| Operación | Generar | Verificar | Prueba |
|---|---|---|---|
| **Arranque** | **0,67 ms** | — | — |
| Emisión (2-de-N custodios) | ~105 ms | ~2 ms | 57.342 B |
| Transferencia | ~620 ms | ~4 ms | 61.966 B |
| Destrucción | ~110 ms | ~2 ms | 54.924 B |
| Auditoría (banda) | ~250 ms | ~1,5 ms | 48.782 B |

**Verificar cuesta el 0,5-0,8% de generar.** El arranque no genera claves:
no hay ceremonia ni secreto que destruir.

> ⚠️ **Esa razón es la de la AUDITORÍA, no la de la transferencia.**
>
> `verify_audit` **solo verifica**: 1,6 ms frente a 274 de generación, un
> **0,58 %**. Es la cifra correcta para el argumento que sostiene —un
> supervisor comprueba sin tocar el estado— pero **estaba atribuida a la
> transferencia**.
>
> Aplicar una transferencia cuesta **28,5 %** de generarla, porque `apply`
> **verifica, muta el árbol y escribe a disco**. No es comparable.
>
> Se detectó ejecutando `cargo test -p zk-ssl --release metrics --
> --nocapture` y comparando con lo publicado. Ver `AUDITORIA.md` §22.

⚠️ **La contención del anclaje de raíz** (`AUDITORIA.md` §123): toda prueba
se ata a la raíz exacta que vio al generarse, así que **dos emisores que no
comparten nada quedan serializados por el anclaje global** — aplicar la
primera invalida la segunda (`StaleState`), aunque toque otra hoja. **El
mecanismo está comprobado** (T5a) y no depende de ninguna constante.

**Y ahora está medido** (§230, banco I.1): cuatro clientes que envían a la
vez, cada uno con sus propias cuentas, **aplican uno y los otros tres son
rechazados** — el **75 %** de las pruebas generadas, a la basura. El nodo
**rechaza barato** —3,1 ms frente a 32 de aplicar— así que el precio no lo
paga él: lo paga quien pierde la carrera. Con **un solo emisor por raíz**
el desperdicio es cero.

⚠️ **Aquí decía «entre 1,5 y 1,9 TPS» y lo llamaba «el techo real del nodo».
Era falso, y por mucho** (§229, §238). Aquella cifra medía **el ciclo
entero en una sola máquina** —un portátil generando las pruebas de las dos
partes— y se atribuía al nodo.

Hecha la resta, de los 1.616 ms que cuestan ocho pagos el nodo trabaja
**65**: el **4 %**. El resto es generar pruebas, que es trabajo del
cliente y en despliegue real ocurre en su máquina.

| | medido |
|---|---|
| **techo del nodo, por RPC** | **248 op/s** (§229, banco H.1: `0,225 + 4,035·n` ms) |
| ciclo completo en un portátil | 4,95 ± 0,15 pagos/s (§222) |
| objetivo de un RTGS | 21 op/s de media — el **8,5 %** de ese techo |

**No falta un factor 2: sobra un factor doce.** Y la cifra la respaldan
tres bancos que no comparten código: el coste fijo por petición sale 0,255
ms en E.2 y 0,225 en H.1; el `apply` sale 3,67 ms en la capa (B.3) y 4,035
por RPC.

⚠️ **Lo que sigue sin medirse**: la latencia por petición, muchos emisores
concurrentes, el nodo contra disco, y **0,216 ms por operación (5,4 %) que
no se explican** — la sospecha es deserializar el DTO, y se anota como
sospecha.

⚠️ **El ~620 s histórico quedó resuelto en `AUDITORIA.md` §130**: era
cifra de otro protocolo (probablemente la vía retirada); el canon vigente
—con dispersión medida— es el de abajo.

⚠️ **Sobre las cifras de tiempo que siguen**: el instrumento repite con
**σ 0,5 %** dentro de una tanda, pero **dos tandas del mismo binario en la
misma máquina difieren un ~9 %** (`AUDITORIA.md` §131). Los tiempos van como
**rango**, y **no son comparables con medidas de otra sesión** a menos del
10 %. La causa de esa deriva **no está investigada**.

**Límites cuantificados**: mil transferencias son **~590 s** de prueba
(un pago son dos: send 353,2 ms · σ 0,6 % + claim 237 ms, protocolo
§89.1, `AUDITORIA.md` §130) y **126,2 MiB** acumulados (envío 65,4 + cobro 63,8 KiB
por pago, medidos).

⚠️ **Un límite que existió, y cómo se fue**: la vía de un paso derivaba
la posición del nullifier del propio nullifier, con colisiones probables
a los ~65.000 pagos. Esa vía está **retirada** y el árbol de
nullificadores con ella (`AUDITORIA.md` §32 y §36): hoy nada los genera.
⚠️ El límite no se resolvió, se evitó — quien distribuya esto lo
recupera entero.

---

## Los cinco paradigmas, comparados

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG |
|---|---|---|---|---|
| Ceremonia | Por circuito | Ninguna | **Ninguna** | Universal |
| Setup | 438 ms | 16,3 s | **ninguno** | 26,3 s + 12,8 s |
| Generación | 422 ms | 4,86 s | **39 ms** | 6,85 s |
| Verificación | 5 ms | 91 ms | **1 ms** | 8 ms |
| Tamaño | **192 B** | 4.096 B | 36,7 KB | 1.008 B |
| Post-cuántico | No | No | **Sí** | No |

**Se eligió STARK descartando Groth16**, que es más rápido y produce
pruebas 320 veces más pequeñas. El motivo: Groth16 exige una ceremonia
cuyos participantes, si coluden, pueden **crear dinero sin dejar rastro**.
Es la única decisión del proyecto tomada contra los números.

**Nova/folding** se midió aparte (~250 ms por transacción, constante) y se
descartó para la capa: usa curvas y exige ceremonia.

---

## Orden de lectura

| Si eres… | Empieza por |
|---|---|
| Alguien con 5 minutos | [`RESUMEN_EJECUTIVO.md`](./RESUMEN_EJECUTIVO.md) |
| **Un revisor de seguridad** | [`AUDITORIA.md`](./AUDITORIA.md) |
| Interesado en la comparativa | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| Interesado en el diseño | [`ARQUITECTURA.md`](./ARQUITECTURA.md) |
| Interesado en el planteamiento | [`PRINCIPIOS.md`](./PRINCIPIOS.md) |
| **Interesado en la visión y sus consecuencias** | [`VISION.md`](./VISION.md) |
| **Llega desde Zenodo o quiere una visión general** | [`doc/ZENODO.md`](./doc/ZENODO.md) |
| Quiere entenderlo sin ser técnico | [`doc/IDEA_CENTRAL.md`](./doc/IDEA_CENTRAL.md) |
| Quiere saber qué aporta frente a lo que hay | [`doc/APORTACION.md`](./doc/APORTACION.md) |
| Interesado en las implicaciones | [`doc/CONSECUENCIAS.md`](./doc/CONSECUENCIAS.md) |
| **Vas a implementar el protocolo** | [`spec/RPC.md`](./spec/RPC.md) + [`spec/vectors/`](./spec/vectors/) |
| Interesado en la formalización | [`doc/VERIFICACION_FORMAL.md`](./doc/VERIFICACION_FORMAL.md) |

`AUDITORIA.md` incluye una sección con **los puntos donde el autor tiene
menos confianza**. Si vas a mirar el código con intención de romperlo,
empieza ahí.

---

## Reproducir

Requiere Rust estable. Sin instaladores externos ni toolchains aparte.

> ⚠️ **`--release` es obligatorio para `zk-ssl`, no una optimización.**
>
> **Cifras vigentes, medidas el 06-08-2026** (sello §199 y su
> verificador). Las del 31-07 y su historia —por qué depuración falla
> por diseño (grados dependientes del testigo), qué se corrigió y qué
> costó corregirlo— viven en `AUDITORIA.md` §20, §76–§77 y no se
> borran: se marcan.
>
> **En release, los dos crates pasan enteros:**
>
> | | tests | fallan | ignorados (con motivo escrito) |
> |---|---|---|---|
> | `stark-experiment` | **297** | 0 | 10 |
> | `zk-ssl` | **242** | 0 | 3 |
>
> Y **0 warnings** en ambos: el verificador de cada sello los cuenta.

```bash
cargo test -p zk-ssl --release              # la capa: 264 tests (3 ign.)
cargo test -p stark-experiment --release    # los circuitos: 297 tests
cargo test -p zk-ssl --release metrics -- --nocapture
```

El crate de circuitos tiene **28 circuitos con `impl Air`** (los dos
últimos: `circuit_send` y `circuit_claim`, el pago en dos fases, ambos
con su ESPEC ejecutable y compuerta de mutantes — ver «Formalización»
arriba). `python3 tools/check_constraint_layout.py` (guardián v7) barre
los 28, canta **seis censos** en línea limpia y no encuentra colisiones,
desbordes ni ranuras muertas.

La comparativa completa:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
cargo test -p nova-experiment --release --features test-setup -- --nocapture
```

**Los tests de circuito conviene ejecutarlos también en debug**: winterfell
solo valida las restricciones al generar en ese modo, y da el índice y la
fila exactos del fallo.

---

## Qué garantiza el sistema

Sin revelar identidades, saldos ni importes:

| Vía de ataque | Cerrada por |
|---|---|
| Transferir más de lo debitado | Conservación (partida doble) |
| Abrir cuenta con saldo | Apertura siempre a cero |
| Emitir sin autorización | **Dos custodios** demostrados en circuito |
| Emisión encubierta | Suministro público atado en el circuito |
| Superar el tope de emisión | Tope inmutable del ledger |
| Gastar dos veces | Encadenamiento de raíces (orden total del nodo único) |
| Gastar sin ser el titular | Autoridad de gasto |
| **Gastar estando congelada** | No-pertenencia al árbol de congelados |
| Reenviar una operación válida | Encadenamiento de raíces |
| Operar sobre estado corrupto | Verificación de integridad al arrancar |
| **Reescribir el historial** | Registro encadenado de transiciones |

Y para cumplimiento: **revelación selectiva** con tres modos —saldo
exacto, mínimo de reservas, y banda ("estoy entre X e Y")—. El titular
produce la prueba; el supervisor la verifica **sin acceso al ledger**.

---

## Ocho hallazgos

Ninguno aparece en los materiales que comparan paradigmas. Todos surgieron
al construir. Detallados en [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md):

1. **AIR carece de restricciones de copia**, y eso abre un agujero
   silencioso al portar actualizaciones de estado.
2. El campo **Goldilocks es demasiado estrecho para identidades**: 64 bits
   son colisión en 2³².
3. Sin extensión de campo, un **STARK sobre Goldilocks tiene techo de 63
   bits** de solidez.
4. La brecha entre seguridad **conjeturada y demostrable**: 127 bits
   conviven con 29-63.
5. **PLONK-KZG resultó el generador más lento** de los cuatro basados en
   curvas.
6. Solo **dos de seis librerías se defienden del uso inseguro** en código.
7. El **ecosistema PLONK-KZG en Rust son stacks verticales cerrados**:
   seis vías investigadas, cinco rotas.
8. **Un zkVM no es comparable en igualdad de condiciones**, y la cifra que
   lo mide son 3 dependencias frente a 349.

---

## Estado y límites

**No auditado por terceros.** Ninguna cantidad de tests propios lo
sustituye.

### ⚠️ El proyecto tiene una dependencia criptográfica sin auditar

Desde §236 el nodo depende de **`xmss` 0.1.0-pre.0** de RustCrypto, para
firmar las cabezas de época. Es una **pre-release y su propio crate declara
que no tiene auditoría independiente**. Se eligió entre cuatro candidatas
con cinco criterios (`doc/xmss-evaluacion.md`, §235) y la versión va
**clavada con `=`** porque `master` diverge del tag publicado.

Hasta ese sello, todo el camino de producción era hash propio y
`winterfell`. **Ahora hay una familia de supuestos más en el árbol**, y
conviene que se lea aquí y no en un `Cargo.toml`.

### Lo que existe hoy, y lo que le falta para valer

| pieza | estado |
|---|---|
| guardián del índice de firma | **construido** (§234). Contador con `fsync` antes de firmar, y **se niega a arrancar si su `fsync` no persiste** — en `tmpfs` cuesta lo mismo que no hacerlo |
| firmante de cabezas | **construido** (§236). Dominio, versión de formato, y verifica su propia salida |
| **custodia de la clave** | **declarada, no comprobada** (§244). El nodo AFIRMA un modelo con `--custodia` y sólo `fichero` lo puede comprobar; de dónde sale la semilla sigue siendo decisión de despliegue |
| latido | **construido** (§241). Emite cabezas por época cada `--latido` segundos, y con `--latido 0` lo dice en voz alta |
| releer una firma desde sus bytes | **construido** (§243). `zk-ssl-verify` recompone la cabeza y comprueba quién la firmó, sin el nodo |
| testigos | **construido** (§245). El testigo fija la clave la primera vez y **se detiene** ante una vista dividida o un cambio de clave |

⚠️ **Corrección §329 (§247: se cita, no se borra).** Hasta este sello las
cuatro filas de arriba decían **no existe** para la custodia de la clave, el
latido, releer una firma desde sus bytes y los testigos. Las cuatro estaban
rancias: la prosa envejeció mientras el árbol avanzaba, y los sellos que la
desmienten —§241, §243, §244 y §245— ya se citaban en `SECURITY.md`
unas líneas antes de negarlos. Lo que sí sigue faltando está en su sitio,
más abajo y en `SECURITY.md`: un ancla anterior al primer encuentro del
TOFU, y una custodia **comprobada**, no sólo declarada.

⚠️ **La custodia está declarada desde el §244, pero declarada no es
comprobada:** mientras el operador sólo la afirme, el valor probatorio de
la firma depende de creerle.
Que el sistema firme no es que la firma sirva.

Lo que falta, por orden de importancia:

- **Consenso distribuido.** Sin él, el operador ve los saldos y puede
  censurar. La alternativa que este proyecto sí persigue —responsabilidad
  demostrable, al modo de Certificate Transparency— **ya tiene las cuatro
  piezas de arriba (§241, §243, §244 y §245): lo que le falta es un
  ancla anterior al primer encuentro y una custodia comprobada, no sólo
  declarada**.
- **Auditoría externa.**
- **El recibo de admisión** (§121): cuatro cosas independientes apuntan a
  esa pieza, y sigue sin construirse.
- Delegación de la prueba a terceros (verificar firma en circuito).
- Política de caducidad para congelaciones; justificación registrada.

Todo lo demás que falta está enumerado en
[`AUDITORIA.md`](./AUDITORIA.md), sección 4.

---

## Dinero cuántico

Dos afirmaciones distintas, para no mezclarlas:

**Lo que este proyecto ya es**: resistente a un adversario cuántico en su
solidez. STARK/FRI no usa emparejamientos ni ceremonia (solo hashes), y la
autoridad de gasto es conocimiento de preimagen —identidad, salt de hoja y
nullifier derivan de la clave por hash (§117)—, sin firma clásica en la vía
de pago. La evaluación de firmas hash-based para los custodios está hecha
(`doc/xmss-evaluacion.md`). Un ordenador cuántico no falsifica estas
pruebas ni gasta estas cuentas.

**Lo que este proyecto NO es**: dinero cuántico. Ese término nombra otra
cosa —estados cuánticos no-clonables como billetes, donde el doble-gasto
lo prohíbe la física y el intermediario de orden (aquí: el operador; en
general: el consenso) puede retirarse del todo. Esa es la dirección que
`PRINCIPIOS.md` §6.bis señala como horizonte; este proyecto es la
aproximación **clásica** disponible hoy, con ese intermediario mínimo,
medido y con su acuse diseñado (`AUDITORIA.md` §121, §174).

## Publicación

Tres preprints, en su **tercera revisión** (30 de julio de 2026). Las
versiones anteriores siguen accesibles y se citan aquí: **una cifra
publicada que se corrige no se borra, se marca**.

**Comparative Implementation of a Zero-Knowledge Settlement Layer across Five
Proof Systems: Design Findings and Measurements**
DOI: [10.5281/zenodo.21693706](https://doi.org/10.5281/zenodo.21693706)

*Versiones anteriores: [10.5281/zenodo.21683239](https://doi.org/10.5281/zenodo.21683239)
y [10.5281/zenodo.21677737](https://doi.org/10.5281/zenodo.21677737). La primera
publica 59,1 MB por mil operaciones y 17,5 % de aplicar sobre generar: las dos
cifras miden la vía de un paso, **retirada desde entonces** (§31, §32).*

**Provable Compliance without Full Ledger Disclosure — A Zero-Knowledge
Settlement Architecture for Supervisory Audit**
DOI: [10.5281/zenodo.21693709](https://doi.org/10.5281/zenodo.21693709)

*Versión anterior: [10.5281/zenodo.21678396](https://doi.org/10.5281/zenodo.21678396).*

**From Institutional Trust to Verifiable Properties — A Minimal ZK Settlement
Layer and Its Residual Trust Surface**
DOI: [10.5281/zenodo.21693718](https://doi.org/10.5281/zenodo.21693718)

*Versión anterior: [10.5281/zenodo.21679208](https://doi.org/10.5281/zenodo.21679208).*

### Qué corrige la tercera revisión

> ⚠️ **Esta tabla es HISTORICA.** Dice que corrigio *esa* revision respecto
> de la anterior, y sus cifras son las de entonces. **No se actualizan**:
> meterle los numeros de hoy reescribiria lo que se publico. Para el estado
> actual, la tabla de arriba, medida el 31-07-2026.

| Corrección | Antes | Ahora | Ver |
|---|---|---|---|
| Cobertura de la prueba por mutación | 12 circuitos limpios | **11** cubiertos (10 de producción); el informe del duodécimo salió de una traza inválida | §12, §20 |
| Árbol de nullificadores | «se conserva por compatibilidad, es peso muerto» | **retirado** con migración verificada; instantánea v4 que sigue leyendo v3 | §32, §36 |
| Confidencialidad frente al receptor | condicionada a que la vía en dos fases fuera la única | la condición **se cumplió**: es la única vía | §32 |
| Tests de los dos crates | 369 | **375** | — |
| Tests que fallan sin `--release` | 56 | **65** de 174, medido | §20 |

⚠️ **Las referencias cruzadas entre los tres preprints apuntan a versiones
anteriores de sus compañeros**, no a las terceras revisiones. Los enlaces
resuelven y el contenido citado sigue siendo el correcto, pero un lector que
los siga leerá una versión con cifras ya corregidas. Se arreglará en la
próxima revisión de los tres.

## Autoría y licencia

**Angel Toranzo Portela**, 2026.

Licenciado bajo **MIT** o **Apache-2.0**, a elección de quien lo use.

Las dos licencias exigen **conservar el aviso de copyright y el texto de la
licencia** en cualquier copia o trabajo derivado. Apache-2.0 exige además
respetar el fichero [`NOTICE`](./NOTICE).

### Código de terceros

`crates/ceremony/` **no es código original de este proyecto**: procede de
`penumbra-sdk-proof-setup` (Penumbra Labs), bajo la misma licencia dual.
Ver [`crates/ceremony/ATTRIBUTION.md`](./crates/ceremony/ATTRIBUTION.md).

### ⚠️ Sin afiliación institucional

Este proyecto es un trabajo **independiente**. **No está afiliado,
respaldado ni encargado por el Banco Central Europeo, el Eurosistema, ni
ninguna otra institución** pública o privada.

Las referencias al euro digital son al diseño público publicado por esas
instituciones y se citan como contexto de un problema técnico. **Ninguna
afirmación de este repositorio debe leerse como posición de nadie más que
su autor.**
