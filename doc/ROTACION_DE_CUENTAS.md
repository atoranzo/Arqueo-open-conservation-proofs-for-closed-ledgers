<!--
  ROTACION_DE_CUENTAS — un solo diseño para cambiar las claves de una cuenta.
  Nota de diseño para la entrada 52 del BACKLOG, ACEPTADA como análisis en el §799 (⚠️ §799: hasta
  aquí decía «en BORRADOR»), que junta los dos casos abiertos de la 20, la hoja nueva del §117, la
  vista irrotable del §127.5 y el reclamo del B18.2 de CONFIANZA_RESIDUAL.md. Hermana de
  AVISO_EN_ISO20022.md, que le añade un caso. No toca código, ni el cable, ni un circuito: mide lo
  que hay y lo que costaría cada opción, y no elige.
-->

# ROTACION_DE_CUENTAS — cambiar de clave sin diseñarlo tres veces

**Estado**: ACEPTADA (§799), como análisis: el autor la acepta como está y deja abiertas sus
decisiones, de la D-1 a la D-9 del §6, sin elegir ninguna opción. ⚠️ §799: hasta aquí decía
«BORRADOR». Análisis sin código y sin número de RFC. Las opciones del §4 se dejan **sin elegir**:
quién autoriza una rotación, dónde cae la hoja nueva y qué casos entran es del autor (§6). No
cierra la 52 ni la 20.

**Procedencia**: la escribió Claude en una sesión de Claude Code en la
nube, sobre `8c02941` (§789), fuera del paso 4 de [`GENAI.md`](../GENAI.md); la lleva el §791 de
`AUDITORIA.md`. Integrar no es aceptar: la aceptación quedó pendiente hasta el §799, en el que el
autor la acepta como análisis, con sus decisiones abiertas. ⚠️ §799: hasta aquí decía «la
aceptación queda pendiente». ⚠️ §792: sus citas al `BACKLOG.md` por línea pasaron a citas por
entrada, la regla v5 de `tools/verificar_citas.py`.
⚠️ §802: el código del §802 movió `lib.rs`, `accounts.rs`, `recovery.rs` y `persistence.rs`, y sus
36 citas a esos ficheros se corrigieron a la línea de hoy: cada una apunta a lo que apuntaba.

**Convención**: ✅ medido en este árbol, con fichero:línea · 📐 derivado de la lectura del código, sin
ejecutarlo · ⚠️ matiz · ❓ no medido.

---

## 0. Por qué un solo diseño

El §134.1 lo dejó escrito (`AUDITORIA.md:10982-11004`): tres decisiones difirieron su coste a la
rotación —el §117, «rotar la clave implica nueva hoja»; el §127.5, la clave de vista «es irrotable
sin rotar la de gasto»; y el B18.2, cuyo reclamo de migración «ES una rotación con hoja nueva»—, y
la 20 tiene dos casos abiertos más. «O se unifican … en UN solo diseño de rotación, o la rotación se
diseñará tres veces, incompatibles entre sí.» La 52 lo recoge con su alcance: unificar los dos de la
20, el reclamo del B18.2 y los tres costes diferidos, en sesión propia (la 52 del `BACKLOG.md`). La
misma entrada pide leer antes dos cosas: cómo se acopla `recover` con los custodios y que la capa no
tiene tiempo. Son el §1.2 y el §1.4.

## 1. Lo que hay hoy, medido

### 1.1 Las claves, y lo que cuelga de cada una

| clave | lo que cuelga de ella | quién la cambia hoy |
|---|---|---|
| **de gasto** `sk` | todo derivado y nada guardado: la identidad (`crates/stark-experiment/src/native.rs:25-48`), el nullificador (`native.rs:67-83`), la sal de la hoja (`native.rs:102-122`) y la clave de vista con su `view_id` (`native.rs:134-185`); abrir guarda la identidad, la sal y el `view_id` ya derivados (`crates/zk-ssl/src/accounts.rs:170-248`) | dos custodios, con `apply_recovery_delegated` (`crates/zk-ssl/src/recovery.rs:52-193`); el titular, no |
| **de custodio** | `custodian_set_root` (`crates/zk-ssl/src/lib.rs:964-974`) | la gobernanza, con `apply_governance_delegated` (`crates/zk-ssl/src/governance.rs:104`) |
| **de gobernanza** | `governance_set_root`, «INMUTABLE» (`lib.rs:975-983`) | nadie: cambiarla es crear un libro nuevo (`governance.rs:18-25`, `:30-34`) |
| **de reposo del libro** | `LedgerKey`, de la frase que el operador da al arrancar (`lib.rs:1056-1062`; `crates/zk-ssl/src/crypto.rs:16-26`) | nadie cambia la frase (§2, caso 5) |
| **del keystore del wallet** | la frase del fichero (`crates/zk-ssl-sdk/src/keystore.rs:1-27`) | `save` con otra frase (`keystore.rs:262`) |
| del firmante del nodo | las cabezas, con XMSS | el RFC-0015, con sus actas: **fuera** de esta nota, que es de las claves de cuenta |

### 1.2 Cómo se acopla la recuperación con los custodios

✅ Lo que `apply_recovery_delegated` hace (`recovery.rs:52-193`), leído línea a línea:

- **La autoridad son dos custodios distintos del conjunto vigente**: dos pruebas de
  `circuit_threshold_single_nullifier` con el dominio de custodio y contra `custodian_set_root`
  (`recovery.rs:151-171`). Las de gobernanza no recuperan (`governance_keys_cannot_recover`,
  `recovery.rs:458`).
- **Gasta cupo de custodio** después de verificar la autoridad (`recovery.rs:172-174`), y el
  registro lo **deriva**: `usos_custodiados` cuenta cada `Recovery` como un uso
  (`crates/zk-ssl/src/log.rs:520-538`).
- **Sube un contador público**, `recovery_count`, que va en las entradas públicas de la subida
  (`crates/stark-experiment/src/circuit_recovery_climb.rs:325-332`) y en el compromiso que firman
  los custodios, junto con las dos raíces (`recovery.rs:145-149`).
- **La hoja se queda en su posición**: identidad nueva, el mismo saldo y el nonce más uno
  (`recovery.rs:89-96`), con la subida de los dos carriles por el mismo camino
  (`circuit_recovery_climb.rs:1-25`). Recuperar hacia la misma identidad se rechaza antes de
  verificar (`recovery.rs:81-83`), y reenviar los mismos materiales, también (`recovery.rs:287`).
- **La API pide la identidad nueva, no la clave** (`recovery.rs:21-25`, `:60`), y por eso una cuenta
  estrecha puede rotar a una clave ancha (`a_narrow_account_can_rotate_to_a_256_bit_key`,
  `recovery.rs:223`), que el propio test declara **no soberana**: «exige DOS CUSTODIOS»
  (`recovery.rs:217-222`).
- **No está en el cable ni en los vectores**: `spec/RPC.md` no tiene ningún método de recuperación
  (su tabla, `spec/RPC.md:76-125`), y `grep -ril "recover\|recuper" spec/vectors` sale vacío. Lo que
  sí está fuera de la capa es el sello: el reverificador recompone el compromiso de una recuperación
  de la era 1 con esas mismas raíces y contadores
  (`crates/zk-ssl-verify/src/reverificacion.rs:178-198`).
- **Lo que no resuelve la recuperación**, dicho en su circuito: la rotación voluntaria «sirve para
  higiene, pero no para el compromiso: si el atacante la tiene, también puede rotar, y ganaría la
  carrera» (`crates/stark-experiment/src/circuit_recovery.rs:10-17`); y si el atacante gasta antes
  de que se aplique, el dinero se va (`circuit_recovery.rs:58-60`).

### 1.3 Lo que la recuperación deja viejo: la costura 49-A↔52

✅ **El `view_id` y la sal se copian** de la clave vieja (`recovery.rs:97-110`), y la copia de la sal
no es solo de la capa: la subida tiene **una sola** columna de sal para los dos carriles, «LA COPIA»
(`circuit_recovery_climb.rs:72-77`; su `build_trace`, `:180-189`, «la rotación es de la costura 52,
fuera de alcance»). La sal la impone el circuito; el `view_id`, que no está en la hoja
(`lib.rs:752-767` frente a `native_leaf_salted`, `crates/zk-ssl-hash/src/lib.rs:205-212`), no lo ata
ninguna prueba.

📐 Lo que se sigue de ahí, leído y no ejecutado; la recuperación no está en el cable, así que hoy
ningún nodo lo alcanza:

- **a. La vista.** El nodo exige la clave de vista para la vista, los materiales de envío y los de
  cobro (`exige_credencial`, `crates/zk-ssl-node/src/main.rs:2446-2451`; `:3699-3707`;
  `:3789-3798`), y la compara con el `view_id` guardado (`crates/zk-ssl/src/client.rs:123-133`).
  Tras una recuperación ese `view_id` es el de la clave vieja: la nueva no pasa la puerta y la vieja
  sí. `doc/ecst/VERIFICACION.md:1323` ya lo dedujo.
- **b. La sal, y el B18.2.** La clave nueva no re-deriva la sal de su hoja; el titular depende de
  que los materiales se la den (`AccountView.leaf_salt`, `client.rs:66-75`). El reclamo de migración
  «exige solo la clave — exactamente la propiedad T2b» (`doc/CONFIANZA_RESIDUAL.md:143-146`), y en
  una cuenta recuperada esa propiedad ya no vale: la sal solo la re-deriva la clave vieja.
- **c. La privacidad frente a la clave vieja.** Quien tenga la clave vieja tiene la sal, y con ella
  la hoja vuelve a ser diccionariable desde un camino, como antes del salt (la medida de la 50 del
  `BACKLOG.md`).
- **d. Los pendientes en vuelo.** El compromiso ata la identidad del receptor
  (`crates/zk-ssl/src/two_phase.rs:925`) y el cobro exige que la clave derive la identidad de la
  cuenta (`two_phase.rs:1229-1231`). Un pendiente enviado a la identidad vieja antes de rotar no lo
  cobra la clave nueva, y la cuenta rotada ya no tiene esa identidad; el reembolso del emisor es
  `apply_refund` (`two_phase.rs:657-697`), y qué hacer con ellos es la D-5. Ningún test lo ejerce.
  ⚠️ §802: ✅ ahora sí, medido y no deducido. `la_clave_nueva_no_cobra_el_pendiente_hacia_la_vieja`
  (`crates/zk-ssl/src/accounts.rs`, al final): tras recuperar X→Y, la clave nueva genera la prueba de
  cobro y aplicarla da `StaleState`, sin mover el saldo ni el pendiente. Y lo que esta nota no vio: la
  clave VIEJA sí lo cobraba, abriendo otra cuenta con la identidad retirada X —medido en privado: cobró
  7.000—. Desde el §802 abrir rechaza una identidad que ya tiene cuenta o que una recuperación retiró
  (`IdentityUnavailable`; `SECURITY.md` §3.13), y el pendiente solo vuelve al emisor por `apply_refund`
  al caducar —salvo un duplicado que la clave vieja abriera antes del §802, que es el residuo de
  `SECURITY.md` §3.13—: `el_robo_tras_una_recuperacion_no_abre_y_el_pendiente_vuelve_al_emisor`. Ni la
  recuperación ni el reembolso están en el cable.
- **e. El test que la costura cita no existe.** `recovery.rs:105` remite a
  `recovery_deja_view_id_viejo`, y `AUDITORIA.md:12774-12777` lo da por hecho; el grep no encuentra
  su definición en ningún crate. `doc/ecst/VERIFICACION.md:1323-1324` ya lo dijo.

⚠️ **Y uno de paso**: el doc del contador público de recuperaciones (`lib.rs:987-993`) está encima
del campo `frozen`, no del `recovery_count` (`lib.rs:1025`). No cambia nada; se dice para que no se
lea mal.

### 1.4 La ausencia de tiempo

✅ La capa rota los custodios **por uso**: `custodian_uses` sube con cada intervención y al llegar al
cupo los custodios dejan de actuar hasta que la gobernanza rote el conjunto, «esta capa no tiene
noción de tiempo» (`lib.rs:1026-1037`; `governance.rs:69-79`). El cupo no lo protege ninguna
autorización (`governance.rs:42-56`).

✅ **El único reloj del código de plazos es `log.len()`**: `let now = self.log.len()` en la
des-emisión y en el reembolso (`apply_deissue`, `two_phase.rs:441`; `apply_refund`, `:676`), contra
el `refund_ttl` de la v1 y contra el `delta` de la v2 (`:445` y `:459` en la primera, `:680` y
`:694` en la segunda). El §119.3 dejó dicho que «`seq` cuenta operaciones, no tiempo» y que el plazo
se cuenta en cabezas firmadas (`AUDITORIA.md:9676-9693`); el código no lee las cabezas como tiempo.

📐 Consecuencia para cualquier opción: una rotación **con ventana** —que la clave vieja o los
custodios puedan impugnarla durante un plazo— hereda o el contador de operaciones, que en un libro
quieto no avanza, o el reloj de cabezas, que no está cableado a la capa. Sin ventana, nada de esto
aplica.

### 1.5 Lo que cuesta tocar una AIR, una operación o el cable

✅ Censado con el nombre de la subida, `recovery_climb` / `RecoveryClimb`, por todo el árbol:

- **La AIR**: `crates/stark-experiment/src/circuit_recovery_climb.rs`; su réplica en la máquina de
  hoja (`doc/spec-maquina-de-hoja.md:77` y `:105-106`), su intérprete en
  `doc/fv/interprete_recovery_climb.py` y su censo en `tools/check_constraint_layout.py:650-651` y
  `:969`.
- **La capa y sus pruebas**: `recovery.rs`, `crates/zk-ssl/src/tests_support.rs:504-529`, las
  llamadas de `crates/zk-ssl/src/tests.rs` y la fila de `RecoveryClimbAir` en la suite de lo que
  revela una prueba (`crates/zk-ssl/src/instrumento_revela.rs:1023-1030` y `:1060`).
- **El mando**: `crates/zk-ssl-cli/src/commands.rs:374`.
- **Un código de operación nuevo**: una línea `REGISTRO` en
  `crates/zk-ssl-hash/src/lib.rs:1708-1745`, su fila en `spec/NUCLEO.md:228-231` y lo que vigila
  `tools/check_nucleo.py`.
- **Un `OpKind` nuevo**: `log.rs:117` y `:136`, el recuento de `persistence.rs:634`, la derivación
  del cupo (`log.rs:520-538`), la tabla de clases del reverificador (`reverificacion.rs:30-52`) y
  los `kind` del cable (`spec/RPC.md:102-105`), que ya es protocolo.
- **Un método nuevo**: `spec/RPC.md`, `spec/openrpc.json`, vectores y RFC (`spec/rfc/PROCESO.md:3-5`
  y su regla 4, `:27-28`).
- **El tren**: la marca v2 del RFC-0018 pondrá en cada prueba su familia y la versión de su AIR, y
  tocar la AIR sube esa versión; hoy la D-G está escrita y sin construir
  (`spec/rfc/0018-el-tren-0-5.md:164-166` y `:221-234`).

## 2. Los casos que la 52 junta, uno a uno

| | caso | de dónde | qué falta, medido |
|---|---|---|---|
| 1 | **hoja nueva** | §117 (`AUDITORIA.md:9569-9625`); «rotar clave implicará nueva hoja» (`native.rs:117-118`) | la segunda sal en la subida (`circuit_recovery_climb.rs:72-77`) |
| 2 | **vista** | §127.5 (`AUDITORIA.md:10487-10499`); «solo rota rotando la clave» (`native.rs:143-146`) | traer el `view_id` nuevo con la rotación (`recovery.rs:97-106`), y decidir qué lo ata (§3, N3) |
| 3 | **reclamo del B18.2** | `doc/CONFIANZA_RESIDUAL.md:471-477` y `:565` | todo: el circuito, la disponibilidad de datos del B18.1 (`:448-458`) y la declaración del B18.3 |
| 4 | **gobernanza inmutable** | la 20 | por diseño, nada; el coste es que una clave de gobernanza comprometida lo está para siempre. Lo fija `the_governance_set_survives_restart` (`tests.rs:1176`), y la raíz va en `params_digest` de cada cabeza (`lib.rs:890-898`) |
| 5 | **recifrado** | la 20 | ver abajo |
| 6 | **la clave del aviso cifrado**, si el B11.2 sale | [`AVISO_EN_ISO20022.md`](AVISO_EN_ISO20022.md) §6.1 | hoy no existe: ninguna cuenta tiene una clave a la que cifrar (`lib.rs:752-767`; `spec/RPC.md:111`) |

✅ **El caso 5 medido, que ya no es el de la 20.** El grep de la 20 sigue vacío, pero desde el §702
(RFC-0001) **hay un re-sellado de todo el libro**: `migrar_a_kdf_v2` abre cada valor con la clave v1
y lo vuelve a sellar con una v2 de sal nueva, en un solo lote de sled
(`crates/zk-ssl/src/persistence.rs:1012-1056`); y el keystore se migra al abrirlo
(`keystore.rs:11-21`). Es con la **misma** frase: cambiar la frase del libro sigue sin existir. Y
hay una medida que la 20 no tenía: **el nodo no cifra en reposo**. Abre el libro con
`SovereignLayer::open` (`crates/zk-ssl-node/src/main.rs:2258-2261`), que llama a `open_encrypted`
sin clave (`persistence.rs:20-37`); `SECURITY.md:413-417` lo declara. Para el nodo no hay frase que
cambiar.

## 3. El núcleo común: lo que cualquier opción comparte

📐 La transición es una: **la hoja `(id, saldo, nonce, sal)` de una posición pasa a
`(id', saldo, nonce + 1, sal')`, con `view_id'` en el registro, sin mover saldo**. La subida de hoy
es esa transición con `sal' = sal` forzado. Cualquier opción necesita estas cuatro piezas:

- **N1. La sal nueva, en la AIR.** Una segunda columna de sal para el carril B: el ancho de la traza
  pasa de 43 a 47 (`circuit_recovery_climb.rs:76-77`), y la longitud no se mueve, porque la
  envoltura de los dos carriles va en el mismo ciclo (`CYC_SALT`, `:92-105`; la fila final es la 279
  de 512). Con su test de mutación, que la casa exige a cada restricción de la sal
  (`doc/spec-maquina-de-hoja.md:88-95`).
- **N2. El `view_id` y la sal nuevos, en lo que se presenta.** Hoy `apply_recovery_delegated` recibe
  solo `new_public_id` (`recovery.rs:52-61`). Las tres derivaciones las haría el cliente, como al
  abrir por el cable (`spec/RPC.md:111`; `accounts.rs:205-209`).
- **N3. Qué ata a cada una.** La sal nueva queda atada por `root_new`, porque va en la hoja. El
  `view_id` nuevo no va en ninguna hoja: solo lo ataría meterlo en el compromiso de la autorización,
  que hoy son las raíces y los contadores (`recovery.rs:145-149`). Eso cambia el sello de la
  entrada: la era 1 del reverificador lo recompone con los parámetros de hoy
  (`reverificacion.rs:178-198`), y la era 2, que lleva el compromiso en la entrada
  (`reverificacion.rs:155-167`), es como la capa ya escribe (`recovery.rs:184`).
- **N4. El cliente.** El `Wallet` del SDK guarda una sola clave de gasto
  (`crates/zk-ssl-sdk/src/lib.rs:93-95`) y el keystore comprueba al cargar que esa clave deriva la
  identidad guardada (`keystore.rs:25-27`): una cuenta rotada es otro fichero, y nada en el SDK
  lleva de uno a otro.

Lo que distingue a las opciones es **quién autoriza** y **dónde cae la hoja nueva**.

## 4. Las opciones, sin elegir

### R1. La recuperación, completada — custodios, misma posición

- **Autoriza**: dos custodios, como hoy.
- **Cubre**: los casos 1 y 2 por la vía de custodia, y cierra la costura del §1.3 (a, b y c).
- **No cubre**: la rotación soberana; cada rotación gasta cupo (`recovery.rs:172-174`) y sube el
  contador público, así que una rotación de higiene se cuenta como una intervención; los casos 3 a
  6; los pendientes en vuelo (§1.3 d) siguen fuera del alcance de la clave nueva. ⚠️ §802: y desde el
  §802 recuperar hacia una identidad con cuenta o retirada se rechaza (`IdentityUnavailable`), antes de
  verificar: deshacer X→Y→X tampoco se puede, y una R1 que quiera volver a una clave anterior necesita
  otra regla. Y quien conozca la identidad nueva antes de que se aplique puede ocuparla abriendo una
  cuenta con ella: la recuperación sale entonces con `IdentityUnavailable` (`SECURITY.md` §3.13).
- **Coste**: N1 a N4 sobre la subida y la capa; los ficheros del §1.5 hasta «El mando». **Cero
  vectores y cero cable**: la recuperación no está en ninguno. Rompe los bytes de la prueba de
  subida, así que va en un tren.
- **Tiempo y carrera**: sin ventana; la carrera de `circuit_recovery.rs:58-60` sigue.

### R2. Rotación propia — la clave vigente, misma posición

- **Autoriza**: el titular, que prueba conocer `sk` como lo prueban el envío y el cobro
  (`two_phase.rs:1231`, y en el circuito).
- **Cubre**: los casos 1 y 2 para la higiene, y para una **clave de vista** comprometida: hoy el
  titular «no puede recuperar la privacidad de lectura sin cambiar de cuenta»
  (`AUDITORIA.md:10493-10494`).
- **No cubre**: el compromiso de la clave de gasto, porque el atacante rota antes
  (`circuit_recovery.rs:10-17`); los casos 3 a 6.
- **Coste**: una AIR nueva, o la subida con un modo: la subida no tiene sección de clave
  (`circuit_recovery_climb.rs:22-25`). **No puede reutilizar `OpKind::Recovery`**: el registro
  derivaría un uso de custodio que no hubo (`log.rs:520-538`) y el contador público mezclaría
  intervenciones con rotaciones (`lib.rs:987-993`). Pide un `OpKind` y, si va al cable, un método,
  sus vectores, su RFC, su fila en la suite del RFC-0009 y su familia en el registro del RFC-0018
  (§1.5). Y una decisión sobre la cuenta congelada: la subida no tiene fase de congelados
  (`circuit_recovery_climb.rs:42-45`).
- **Tiempo y carrera**: sin ventana; el reenvío lo para el nonce más uno con las raíces, como en la
  recuperación (`recovery.rs:287`).

### R3. Rotación precomprometida — la clave siguiente, el patrón del RFC-0015

- **Autoriza**: conocer una clave siguiente cuyo compromiso la cuenta fijó antes. Es el patrón con
  que el RFC-0015 rota al firmante del nodo, que «no arranca con una clave que nadie comprometió»
  (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:1` y `:38`).
- **Cubre**: los casos 1 y 2, y **el compromiso de la clave de gasto sin custodios**, si la
  siguiente se guardó aparte: quien roba `sk` no tiene la comprometida.
- **No cubre**: los casos 3 a 6; perder la siguiente devuelve a los custodios.
- **Coste**: el compromiso tiene que estar donde el operador no lo pueda cambiar. **En la hoja**,
  cambia la preimagen en las doce AIR que la rehacen —`circuit_audit`, `circuit_banda`,
  `circuit_burn`, `circuit_claim`, `circuit_claim_v2`, `circuit_credit_climb`, `circuit_mint`,
  `circuit_mint_climb`, `circuit_recovery` (que la capa ya no llama), `circuit_recovery_climb`,
  `circuit_send` y `circuit_send_v2`, según el grep de `leaf_salt` en
  `crates/stark-experiment/src/`—, la función del núcleo (`crates/zk-ssl-hash/src/lib.rs:205-212`)
  con su KAT nuevo junto a `spec/vectors/nucleo/native_leaf_salted.json`, y una migración de todas
  las hojas: la clase del B13/B14. **Solo en el registro**, la atadura es la palabra del operador,
  que guarda los registros (`lib.rs:745-750`). Más todo lo de R2.
- **Tiempo y carrera**: no necesita ventana. El RFC-0015 la evita porque quien juzga las actas es el
  testigo, no la capa.

### R4. Traspaso — una cuenta nueva y un pago, sin código

- **Autoriza**: el titular, con un envío y un cobro.
- **Cubre**: los casos 1 y 2, porque la cuenta nueva abre con la sal y la vista de la clave nueva
  (`accounts.rs:170-178`). Es **la única** en la que los pendientes en vuelo se siguen cobrando:
  la cuenta vieja conserva su identidad, y la clave vieja los cobra en ella
  (`two_phase.rs:1229-1231`) para volver a traspasarlos (en las demás, §1.3 d).
- **No cubre**: el compromiso, porque el atacante traspasa antes y a su cuenta; los casos 3 a 6. La
  cuenta vieja sigue viva con la clave vieja y sigue recibiendo; el registro IBAN del puente, que es
  local (`crates/zk-ssl/src/iso.rs:297-299`) y no se persiste
  (`crates/zk-ssl/src/migration.rs:46-47`), hay que reapuntarlo a mano.
- **Coste**: **ningún fichero**. Lo operativo: dos pruebas por rotación, las de un pago; una plaza
  del tope de cuentas (`lib.rs:1043-1054`), que abrir no autoriza (la 8 del `BACKLOG.md`); el
  enlace viejo → nuevo, a la vista de quien vea las dos mitades (`spec/RPC.md:344-347`) y del
  operador. Una cuenta congelada no puede: el envío lo comprueba (`two_phase.rs:859`).
- **Tiempo y carrera**: sin ventana.

### R5. El reclamo como rotación — el B18.2 generalizado

- **Autoriza**: la clave del titular, abriendo su hoja bajo una raíz firmada —la `H*` del siniestro
  o una cabeza viva— (`doc/CONFIANZA_RESIDUAL.md:471-477`); los custodios, si se quieren, como
  respaldo.
- **Cubre**: el caso 3 por construcción, y el 1 y el 2 si la hoja nueva se escribe con identidad,
  sal y vista nuevas. Y el 4 **de lado**: «cambiarla exige crear un ledger nuevo» (`lib.rs:975-983`)
  deja de ser fatal para los titulares si cada cuenta puede pasar por reclamo a un sucesor con otra
  raíz de gobernanza (G2, abajo).
- **No cubre**: el caso 5. Es la más cara.
- **Coste**: un circuito nuevo de la clase `circuit_claim`; un registro de posiciones ya reclamadas
  en el sucesor —el precedente es el mapa `claimed` del árbol de pendientes
  (`crates/zk-ssl/src/pending.rs:204`)—; el B18.1 (`doc/CONFIANZA_RESIDUAL.md:448-458`), el B18.3 y
  el B12.1; método, vectores, RFC y segunda implementación. Y su precondición, que el titular
  re-derive la sal con su clave, **falla en las cuentas recuperadas** mientras no exista N1 (§1.3
  b).
- **Tiempo**: la ventana de reclamos es «política, no parámetro»
  (`doc/CONFIANZA_RESIDUAL.md:487-490`): necesita reloj, y hereda el §1.4.

### La gobernanza y el recifrado, que no son claves de cuenta

- **G0. La gobernanza, inmutable** como hoy: coste cero, y el declarado de la 20.
- **G1. Que la gobernanza se rote con su propio umbral**: una vía como `apply_governance_delegated`
  (`governance.rs:104-169`) con un código de operación nuevo (§1.5), la raíz mutable y persistida, y
  `params_digest` cambiando entre cabezas (`lib.rs:890-898`). ❓ Qué hace el testigo cuando
  `params_digest` cambia entre dos cabezas no está medido aquí; rotar el conjunto de custodios ya lo
  mueve hoy. La circularidad se traslada otra vez: dos claves de gobernanza comprometidas rotarían
  la gobernanza a las suyas.
- **G2. Por R5**: un libro nuevo con otra raíz y los reclamos de todas las cuentas.
- **K0. El recifrado, como hoy**: solo la biblioteca, y el nodo en claro.
- **K1. Cambiar la frase**: generalizar `migrar_a_kdf_v2` a dos frases (`persistence.rs:1022-1056`);
  en el nodo, antes, cablear el cifrado (`SECURITY.md:413-417`); en el keystore, `load` y `save` ya
  bastan (`keystore.rs:262` y `:280`). Ningún circuito y nada en el cable, como el RFC-0001, que no
  movió `zkssl/0.4` (`spec/rfc/0001-el-kdf-de-reposo.md:12`). ✅ **No comparte pieza con las opciones
  de cuenta** salvo `persistence.rs`, que persiste los registros: si entra en el diseño único es por
  calendario, no por mecanismo.

### V. Solo la vista, con un contador

📐 Una variante del caso 2 que no rota la clave de gasto: un contador **público** en la derivación de
la vista. No es un secreto nuevo —el §127.3 descartó el secreto independiente por ser el modo de
pérdida fatal que el §117 rechazó (`AUDITORIA.md:10455-10469`)—, pero cambia esa decisión, y fijar
el `view_id` nuevo sigue pidiendo que lo autorice `sk` con una prueba: es la clase de coste de R2
sin tocar la hoja.

## 5. La matriz

| | 1 hoja | 2 vista | 3 B18.2 | 4 gobernanza | 5 recifrado | 6 aviso | robo de `sk` | AIR | cable |
|---|---|---|---|---|---|---|---|---|---|
| R1 | sí | sí | — | — | — | — | sí, con custodios | la subida | no |
| R2 | sí | sí | — | — | — | — | no | nueva, o la subida con un modo | sí |
| R3 | sí | sí | — | — | — | — | sí, sin custodios | las doce de la hoja | sí |
| R4 | sí | sí | — | — | — | — | no | ninguna | no |
| R5 | sí | sí | sí | por G2 | — | — | según quién autorice | nueva | sí |
| G1 | — | — | — | sí | — | — | — | ninguna | ❓ |
| K1 | — | — | — | — | sí | — | — | ninguna | no |
| V | — | sí | — | — | — | — | no | ninguna de la hoja; una prueba de `sk` | sí |

El caso 6 no lo cubre ninguna porque todavía no hay clave: si el B11.2 la crea, se rotará con la
opción que se elija aquí, o se derivará de `sk` y rotará con ella (D-9).

## 6. Decisiones abiertas para el autor

- **D-1. Quién autoriza** una rotación de cuenta: los custodios, la clave vigente, una siguiente
  comprometida, o una combinación —por ejemplo, la vigente para la higiene y los custodios para el
  robo—.
- **D-2. Dónde cae la hoja nueva**: en su posición, en otra del mismo libro o en otro libro.
- **D-3. Si la gobernanza entra**: G0, G1 o G2.
- **D-4. Si el recifrado entra o va aparte**: K0 o K1, y si antes se cablea el cifrado en el nodo.
- **D-5. Los pendientes en vuelo** hacia la identidad vieja (§1.3 d): si se aceptan varados hasta el
  reembolso, si la rotación los espera o si la identidad vieja sigue cobrando. ⚠️ §802: la tercera ya no
  puede ser una cuenta nueva con la identidad vieja —una identidad retirada no vuelve a abrir—; solo un
  diseño en el que la cuenta rotada acepte su identidad anterior. Y toda rotación que retire una
  identidad tiene que añadirla al mismo conjunto de retiradas, en el mismo lote.
- **D-6. El contador**: si una rotación propia cuenta en `recovery_count` o tiene el suyo.
- **D-7. La cuenta congelada**: si puede rotar, y si rotar puede sacarla de su posición.
- **D-8. El tren**: el 0.5 del RFC-0018 u otro.
- **D-9. La clave del aviso cifrado**, si el B11.2 sale: derivada de `sk`, y rota con ella, o
  independiente, que es el secreto nuevo que el §117 y el §127.3 no quisieron.

## 7. Lo que esta nota no hace, y el siguiente paso

No toca código, ni el cable, ni un circuito; no gasta número de RFC; no elige; no cierra la 52 ni la
20. Los pasos que siguen necesitan compilar, y esta sesión no compila:

1. el test que `recovery.rs:105` cita y no existe: una recuperación deja el `view_id` viejo, y la
   clave nueva no pasa `account_view_authenticated`;
2. el pendiente en vuelo tras una recuperación (§1.3 d), medido con un test y no deducido; ⚠️ §802:
   hecho, con el robo que destapó cerrado (§1.3 d);
3. de la opción que se elija, su test de mutación sobre la sal nueva (N1).
