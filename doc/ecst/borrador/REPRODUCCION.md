# Reproduccion ejecutada el 2026-09-28 (TRABAJO EN CURSO)

> ⚠️ Borrador de trabajo, no un resultado publicable. Lo ejecuto un asistente de IA generativa
> (Claude, de Anthropic) en un contenedor efimero, no el autor en su maquina: ver `GENAI.md`.
> Las cifras de aqui son de UNA maquina y se dan con su entorno; no sustituyen a las de `AUDITORIA.md`.

Entorno: microVM Firecracker (kernel 6.18.44-fc-v37), 4 vCPU "Intel(R) Xeon(R) Processor @ 2.10GHz", 15 GiB RAM,
raiz ext4 sobre /dev/vda (virtio), /dev/shm tmpfs. rustc/cargo 1.94.1 (2026-03-24). Python 3.
Arbol Arqueo: d531c8016800573f1a6b3e3e4c699a3551da20cd (rama claude/awesome-pasteur-u08nq9 = main en ese commit).
Arbol hbs-state: a960828bf0e810c30b49640c9bcb8d64f9c9557f (clon superficial de github.com/atoranzo/hbs-state).

## Arqueo
- `cargo test --release -p zk-ssl --lib t1_` -> 3 passed (t1_divergencia_localizada, t1_verify_chain_caza_sustitucion,
  t1_cabeza_ata_la_historia), 0 failed. El test usa N=12 entradas y altera la K=5 (constantes en crates/zk-ssl/src/log.rs:1140-1141),
  con "pruebas" sinteticas (bytes b"honesta"/b"mentira"), sin STARK; recompone con chain_digest_v2 desde los campos crudos.
- `cargo test --release -p zk-ssl-guardian` -> 26 passed, 0 failed (incluye en_tmpfs_se_niega_a_operar, en_disco_de_verdad_si_opera,
  con_el_contador_en_uno_y_la_clave_en_cero_falla_cerrada, solo_la_clave_adelantada_no_admite_matiz).
- Verificador independiente `zk-ssl-verify` (compilado en release en este arbol): los vectores del kit de README dan las salidas
  esperadas (posicion-v2 -> 0 VERDE; consumo -> 0 VERDE; rechazo-n-adulterado -> 1 ROJO; rechazo-cons-ausencia-ya-estaba -> 1 ROJO;
  conflicto -> 0 VERDE "DETECCION, no prevencion"; rechazo-conf-camino-no-sube -> 1 ROJO).
- `bash tools/conformidad.sh target/release/zk-ssl-verify [MANIFIESTO]` sobre los ocho catalogos: paquete 70/70, consumo 14/14,
  conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 = 222/222 entradas dicen lo que deben
  (binario 4305941dff1ab547).
- `cargo run --release -p zk-ssl-cli -- simulate --amount 250000` (salida integra en `simulate-2026-09-28.log`; UNA ejecucion, sandbox con claves deterministas, semilla 0xa11ce):
  pruebas STARK reales; emision delegada (mint) con dos custodios: 64.3 KB [417 ms] y 65.3 KB [345 ms]; send 77.6 KB [970 ms];
  claim 76.7 KB [1198 ms]; apply 4-6 ms cada uno; "cadena de transiciones integra (6 entradas)". Es una sola ejecucion en una
  microVM compartida: NO es una medicion con dispersion y no sustituye a las de AUDITORIA.md. "KB" de la CLI = bytes/1024 (crates/zk-ssl-cli/src/trace.rs:254), es decir KiB. Envio+cobro = 77.6+76.7 KiB ~ 158 000 bytes,
  DENTRO de la banda publicada de un pago PUBLICADA_PAGO_MIN_B..MAX_B = 145 953..167 967 bytes (crates/zk-ssl/src/metrics.rs:82-83,
  banda desde S538 con la ocultacion; las cifras 66 739 / 66 692 bytes del comentario de metrics.rs son ANTERIORES a S538 -el esceptico del bloque M las
  situa en el §512, no en la fecha 2026-08-26 que el comentario les pone-: no deben citarse como vigentes. El rango
  MEDIDO por pago antes del margen del 5 % fue 155 337..159 329 bytes, segun refutacion-measures.json, M2a/M2c).

## hbs-state
- `cargo test --release` -> 27 (lib) + 8 (tests/vectors.rs) passed, 0 failed.
- `python3 spec/verify-state.py --vectors spec/state-vectors-v0.3.json --subject ./target/release/hbs-state-subject`:
  familia A N1 12/12, N2 12/12, N3 12/12; familia B 6/6; N0 declara 'seed_derived'; NIVEL ALCANZADO: N3.
- Autocontrol de fsync (fuente de la sonda: `sonda-fsync.rs.txt`): replica literal de check_persistence (hbs-state src/lib.rs:523-556: 20 escrituras de 8 bytes con y sin
  sync_all; rechaza si ratio < 10 Y con_fsync < 20 us; umbrales DECLARADOS en src/lib.rs:97-108), repetida 30 veces por ruta y
  por tanda, dos tandas:
  | ruta | con fsync (us) p50 [min-max] | sin fsync (us) p50 | ratio p50 [min-max] | rechazos |
  | ext4 /dev/vda, tanda 1 | 128.9 [74.7-292.2] | 0.876 | 161.4 [92.6-292.9] | 0/30 |
  | ext4 /dev/vda, tanda 2 | 173.1 [103.6-795.2] | 0.879 | 189.4 [116.9-986.5] | 0/30 |
  | tmpfs /dev/shm, tanda 1 | 0.537 [0.531-1.582] | 0.421 | 1.3 [0.7-3.8] | 30/30 |
  | tmpfs /dev/shm, tanda 2 | 0.536 [0.533-0.732] | 0.421 | 1.3 [0.7-1.8] | 30/30 |
  Limitacion que esto ilustra: en una microVM el guest no puede saber si el host honra el flush hasta el medio fisico; el
  autocontrol solo demuestra que fsync CUESTA algo aqui, que es exactamente lo que declara medir.

# Re-ejecucion del 2026-09-30 sobre `main` 71c5aad (S582)

Mismo tipo de entorno (microVM Firecracker, kernel 6.18.44-fc-v50, 4 vCPU). Arbol: 71c5aad + el commit de
este borrador (71710b2). `main` avanzo 18 commits desde d531c80 (S566-S582: cabeza v6 con raiz de
recepcion, RFC-0010 ACEPTADO, sobre de completitud).

- `cargo test --release -p zk-ssl --lib t1_` -> 3 passed, 0 failed.
- `cargo test --release -p zk-ssl-guardian` -> 26 passed, 0 failed.
- `bash tools/conformidad.sh target/release/zk-ssl-verify [MANIFIESTO]` (binario fea39a053efd7089): paquete 70/70,
  consumo 14/14, conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 y el catalogo
  NUEVO completitud 35/35 = 257/257.
- `simulate --amount 250000` (salida en `simulate-2026-09-30.log`, UNA ejecucion): emisiones 66.8 KiB [328 ms] y
  66.5 KiB [364 ms]; envio 78.9 KiB [1333 ms]; cobro 77.8 KiB [879 ms]; apply 5-6 ms. Envio+cobro = 156.7 KiB
  ~ 160 460 bytes, DENTRO de la banda 145 953..167 967. Frente al 2026-09-28 (77.6 + 76.7 KiB) el tamano
  cambia de una ejecucion a otra, como declara metrics.rs: con la ocultacion cada prueba pesa segun q y la sal.

# El kit publicado frente al arbol de hoy (2026-09-30)

`git worktree add` del tag `arqueo-verify-v0.2.0` (1528943) y `cargo build --release -p zk-ssl-verify` con un
`CARGO_TARGET_DIR` aparte; el binario, contra vectores de `main` en 71c5aad, y el del arbol al lado:

| vector | cabeza | kit v0.2.0 | arbol 71c5aad |
|---|---|---|---|
| `paquete/posicion-v2.json` | v3 | 0 VERDE | 0 |
| `consumo/consumo.json` | v4 | 0 VERDE | 0 |
| `edad/edad-todos.json` | v5 | 1 ROJO «tipo desconocido: edad» | 0 |
| `prenda/prenda.json` | v5 | 1 ROJO «tipo desconocido: prenda» | 0 |
| `pendiente/cobro-inferior-0.json` | v5 | 1 ROJO «tipo desconocido: cobro_pendiente» | 0 |
| `completitud/declarada.json` | v6 | 1 ROJO «tipo desconocido: completitud» | 3 (su salida para «declarada») |
| `completitud/no-resuelta.json` | v6 | 1 ROJO «tipo desconocido: completitud» | 1 (el rojo con nombre que el vector pide) |
| `ancla/ancla-derivada.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, contra `main` en ab79a50) |
| `ancla/ancla-exacta.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, contra `main` en ab79a50) |
| `ancla/ancla-extendida.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, contra `main` en ab79a50) |

Y por lectura: su `main.rs` importa `epoch_digest_v2`, `v3` y `v4`, y nada mas. Falla cerrada: ningun VERDE falso.
