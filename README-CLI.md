# zk-ssl-cli — sandbox y trazador de ZK-SSL

CLI en Rust para probar la lógica de la capa, simular operaciones,
inspeccionar el estado e imprimir trazas detalladas **sin levantar nada**:
ejecuta `zk_ssl::SovereignLayer` de verdad, con pruebas STARK reales, en
memoria o contra un ledger `sled` persistido.

## ¿Y REVM? No, y el motivo importa

REVM es un **ejecutor del EVM**: interpreta bytecode EVM y solo modela el
estado de Ethereum (cuentas, storage slots, opcodes). Su `Inspector` traza
opcodes y frames de CALL, nada más.

Esta capa no tiene EVM: es lógica Rust nativa sobre STARK/FRI
(winterfell). Los nullifiers de umbral de custodios
(`circuit_threshold_single_nullifier`), el árbol disperso de cuentas, el
árbol de pendientes, el flujo en dos fases send/claim y el
`TransitionLog` encadenado **no existen para REVM**: no hay bytecode que
interpretar ni slots donde mirarlos. Usar REVM exigiría reescribir toda
la capa como contratos EVM, perdiendo justo lo que la define (verificación
STARK nativa, sin ceremonia).

El equivalente correcto del `Inspector` de REVM aquí es lo que hace este
CLI: un enum de eventos (`TraceEvent`) emitido desde las fases reales de
la capa, con sumideros intercambiables (trait `Tracer`).

Matiz sobre "los nullifiers": en la capa STARK, el doble gasto de pagos
se cierra con nonce + hoja + árbol de pendientes ("no-pertenencia
demostrable"), no con nullifiers de nota; los nullifiers que SÍ existen
aquí son los anti-replay de las autorizaciones de custodios/gobernanza, y
`simulate` los imprime en cada emisión. Los nullifiers de nota clásicos
(`nullifier.rs`, `persistent_nullifier_registry.rs`) viven en el backend
experimental Halo2, no en la capa.

## Estructura

```
crates/zk-ssl-cli/
├── Cargo.toml
└── src/
    ├── main.rs      # clap (derive) + init de tracing-subscriber (stderr)
    ├── commands.rs  # simulate / trace-tx / inspect-state
    ├── conformance.rs # --emit/--check: los vectores del protocolo (§198)
    ├── sandbox.rs   # motor: fases reales de la capa, instrumentadas
    ├── trace.rs     # enum TraceEvent + trait Tracer (consola / JSONL)
    └── fmt.rs       # hex de Digest vía zk_ssl::store (misma serialización)
```

No hay crate `mychain-core`: el motor de estado ya existe y es `zk-ssl`.
El CLI depende de él con la feature `sandbox` (ver `PARCHES.md`).

## Uso

```bash
# Envío + cobro en dos fases, en memoria, con traza por fases:
cargo run -p zk-ssl-cli -- simulate --amount 250000

# Dejar el pendiente en tránsito (para verlo en inspect-state):
cargo run -p zk-ssl-cli -- simulate --no-claim

# Contra un ledger persistido (se crea si no existe):
cargo run -p zk-ssl-cli -- simulate --ledger ./ledger --amount 100000

# Paso a paso del registro encadenado (demo en memoria si no hay ledger):
cargo run -p zk-ssl-cli -- trace-tx --ledger ./ledger --last 20
cargo run -p zk-ssl-cli -- trace-tx --ledger ./ledger --seq 3

# Estado: raíces, suministro, en tránsito, cabeza del registro, cuentas:
cargo run -p zk-ssl-cli -- inspect-state --ledger ./ledger --accounts

# Conformidad del protocolo: re-ejecuta el escenario canónico y compara
# campo a campo contra los vectores fijados — la 2ª implementación empieza aquí:
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.1.json

# Eventos como JSON Lines (datos por stdout, diagnóstico por stderr):
cargo run -p zk-ssl-cli -- --json simulate | jq .

# Spans y tiempos internos:
cargo run -p zk-ssl-cli -- --log debug simulate
```

Salida típica de `simulate` (abreviada):

```
━━ FASE FUND ━━ emisión delegada de 1000000 a #0: exige DOS custodios
  ✔ prueba STARK generada: 36.7 KB — digest 3fa2…9c01 [812 ms]
  · custodio #1 autoriza — nullifier consumido: 8b1e…
  · custodio #3 autoriza — nullifier consumido: 44c7…
  ✔ aplicado → log#1 Mint  raíz a01b…→77d2…  cadena 5e90… [95 ms]

━━ FASE SEND ━━ #0 → #1, importe 250000: materiales → prueba LOCAL → apply_send
  · materiales listos (pendiente@0) [0 ms]
  ✔ prueba STARK generada: 36.7 KB — digest c3d4… [1204 ms]
  ✔ aplicado → log#3 Send  raíz 77d2…→b6aa…  cadena 09f1… [102 ms]
  el dinero está EN TRÁNSITO: no es del receptor hasta que cobre (§29)

━━ FASE CLAIM ━━ ...
✔ cadena de transiciones íntegra (5 entradas)
── estado del libro mayor ── ...
```

## Decisiones de diseño

- **`trace-tx` lee el `TransitionLog` de la capa**, no un log paralelo: el
  "id de transacción" es el `seq` del registro encadenado, con su
  `proof_digest` y el digest de cadena. Es el mecanismo de auditoría que
  el proyecto ya tiene; el CLI solo lo hace visible y ejecuta
  `verify_chain()`.
- **`send`/`claim` van por la vía de cliente** (`send_materials` →
  `client::prove_send` → `apply_send`): ninguna clave llega a la capa,
  que es el flujo de portada del README del proyecto.
- **Sin `tokio`.** La capa es síncrona y "no hay red ni consenso" por
  diseño. Cuando exista un nodo con JSON-RPC, lo limpio es una feature
  `rpc` (tokio + jsonrpsee) que implemente `Tracer`/consultas contra el
  nodo, no arrastrar un runtime async hoy. Ese nodo **ya existe** desde §197
  (`zk-ssl-node`, axum): este CLI sigue síncrono a propósito, contra la
  capa directa; el camino con red es SDK↔nodo.
- **`Result` en los bordes, enums en el centro**: los `LayerError` de la
  capa suben tal cual; cada rechazo emite `TraceEvent::Rejected` antes de
  propagarse.

## Avisos

- Claves del sandbox **deterministas** (derivadas de `--key-seed`): solo
  para pruebas. Con `--ledger`, reutiliza la misma semilla con la que se
  creó, o `prove_send` fallará con `NotTheAccountHolder`.
- Los parámetros (`--limit`, `--max-supply`, `--max-accounts`) de un
  ledger persistido deben coincidir con los de su creación
  (`ParameterMismatch` si no: es inmutabilidad, no un fallo).
- Las pruebas son reales: cada fase tarda del orden de segundos según
  máquina (las cifras de PERFORMANCE.md aplican).
- Saldos listados con `--accounts`: vista del **operador** por diseño
  (§129 de AUDITORIA.md).
- Escrito contra las firmas actuales de `zk-ssl` (verificadas en el
  código fuente); pasa `cargo check -p zk-ssl-cli` tras aplicar
  `PARCHES.md` — este entorno no tiene red para compilar dependencias.
