# Arqueo: plano de arquitectura v2.0 (Blueprint)

**Diagnóstico, debate y plan por cortes para el motor de libro cerrado: qué optimizar, qué no, y
con qué números**

> **English:** A swarm of 17 agents read the tree at `7d13f26`, checked each other and argued over
> what to change in Arqueo; this is the public synthesis, re-checked on `e69fadd`. Its answer to
> "millions of proofs per second" is no, with numbers: verifying costs 4-5 ms per proof (measured
> in this session), one chain of roots tops out near 1,300 operations/s (estimated), and the RTGS
> target the repository sets is 21-105 operations/s. It proposes an ordered refactoring in cuts,
> the first of which closes eight findings under embargo that this document does not describe.

**Estado**: PROPUESTA. Nada de lo que aquí se propone está construido en el árbol ni medido como
sistema: los bocetos solo se compilaron, y dos se ejecutaron, en una copia desechable (§5.4).
Discutirlo y decidir es del autor: pasos 2 y 3 de [`GENAI.md`](../GENAI.md). Cada
corte del §5.3 es un candidato a bloque, no un bloque.

**Sobre qué commit**: el enjambre leyó `7d13f26` (§631 de [`AUDITORIA.md`](../AUDITORIA.md)).
Este documento se comprobó sobre `e69fadd` (§632). Entre los dos solo cambian
`crates/zk-ssl-medio`, `Cargo.toml`, `Cargo.lock`, `NOTICE`, `tools/canon.sh` (una fila),
`spec/README.md`, el RFC-0013 y cifras de AUDITORIA, BACKLOG, PAPER y PRINCIPIOS. De las citas de
este documento caen en esos cambios el `resolver` de `Cargo.toml`, que pasa a la línea 47, y los
recuentos del lock, que se dan en los dos commits: de 329 a 335 nombres de paquete por las
dependencias del medio del ancla (§632) y, con el mismo script, de 104 a 102 nombres que solo
existen por los comparativos y de 116 a 118 paquetes en la clausura del cable. Entra en el árbol
sobre `f7aad05` (§635): el §633, el §634 y el §635 tocan el kit, el cli, el lock y el canon, y de las
citas de este documento solo mueven tres, ya puestas al día (`crates/zk-ssl-verify/src/main.rs`,
`crates/zk-ssl-verify/src/lib.rs` y `crates/zk-ssl-cli/src/prenda.rs`). Los recuentos del lock se
quedan en los dos commits que nombran.

**Cómo se hizo**: con Claude Code, en una sesión en la nube y no en la máquina del autor.
Diecisiete agentes. El documento sigue tres fases: **diagnóstico** (§3: cuatro especialistas,
de criptografía y ZK, rendimiento y estructuras de datos, seguridad, y arquitectura y API, y
cuatro verificadores adversariales, uno por especialista, con veredicto CONFIRMADO, MATIZADO o
REFUTADO sobre cada cita y cada cifra), **debate** (§4: cinco debates cruzados, D1-D5, y cuatro
réplicas, en las que cada autor CONCEDE, DEFIENDE o ENMIENDA y deja una propuesta final) y
**síntesis** (§5).
La sesión midió además la línea base en su propia máquina (§2). Este texto lo redactó el
orquestador del enjambre, un agente de Claude Code, a partir del expediente
público del enjambre (el expediente sin lo embargado) y del árbol. Después lo revisaron cinco
agentes más de la misma sesión (citas, cifras, embargo, bocetos y completitud), un corrector
aplicó sus correcciones y los bocetos se compilaron en una copia desechable de `e69fadd` (§5.4).
Ninguna persona lo revisó antes de escribirse. La aceptación que describe
[`GENAI.md`](../GENAI.md) queda pendiente, como en
[`firma-corta-evaluacion.md`](firma-corta-evaluacion.md).

**Embargo** ([`SECURITY.md`](../SECURITY.md) §5: los fallos de solidez se reportan en privado
antes de divulgarse). **Ocho hallazgos están en embargo: cuatro P0, tres P1 y un P2.** Se entregaron
al autor en privado. Tres de ellos se reprodujeron con tests en la sesión. **El corte 0 del plan
(§5.3) es cerrarlos.** Este documento no los describe, y por eso tampoco da su componente, su
mecanismo ni su arreglo. Allí donde el expediente público mezclaba un hallazgo publicable con uno
embargado, se publica solo lo que se sostiene sin el segundo, y se dice.

**Convención**: **MEDIDO** = hay una medida, con su fuente (la sesión, o el §N del registro) ·
**ESTIMADO** = derivado, con su método al lado · **SUPUESTO** = sin método que lo sostenga todavía ·
⚠️ = límite · **pre-§538** = medido antes de que el §538 encendiera la ocultación (zkssl/0.4);
esas cifras ya no son la base vigente, aunque sigan siendo correctas como registro.

**Identificadores**: REND-n (rendimiento), ZK-n (criptografía), SEC-n (seguridad) y ARQ-n
(arquitectura) son los de este documento. La correspondencia con el expediente del enjambre la
tiene el autor.

---

## 1. Resumen ejecutivo

**La pregunta de partida**, en el encargo genérico del enjambre: cómo llevar Arqueo a «millones de
pruebas de conservación por segundo», con Pedersen, vector commitments, SNARK, Bulletproofs,
zero-sum proofs o recursión si hacía falta.

**Respuesta corta: no, y no hace falta.** Con números:

| | valor | etiqueta y fuente |
|---|---|---|
| verificar una prueba oculta, por núcleo | **3,98-4,93 ms** → 203-251 verificaciones/s | MEDIDO, sesión (§2, `el_coste_de_verificar_una_prueba`, `crates/zk-ssl/src/metrics.rs:247`) |
| generar una prueba oculta, un hilo | **817-865 ms** (medianas de cobro y envío) → ≈1,2 por segundo | MEDIDO, sesión (`remedicion_89_1`, `crates/zk-ssl/src/metrics.rs:1046`) |
| un nodo hoy, operación suelta, en memoria | **≈145-185 op/s** | ESTIMADO: 1.000 / (apply de la capa en memoria, 4,8-6,2 ms, MEDIDO en la sesión, + ≈0,59 ms de RPC: 0,225 ms fijos por petición y 0,365 por operación, §229, pre-§538 y en otra máquina) |
| un nodo hoy, operación suelta, con disco | **≈110-130 op/s** | ESTIMADO: lo anterior + 2 × 0,907 ms de fsync (§234) + 0,55 ms de flush de sled (§204), medidos en otra máquina |
| un nodo con el plan (lote, 4 núcleos), en memoria | **≈460-530 op/s** | ESTIMADO: 0,77 ms en serie + (3,98-4,93 ms de verificación de la sesión + ≈0,5 ms de `root_with`) / 4 da ≈470-530; la réplica de rendimiento da ≈500 en memoria y ≈460 con disco y el commit por lote, y D4 ≈530 con la caché de ZK-8. Sin medir: el corte 1 lo convierte en banda |
| **una cadena de raíces (un shard), núcleos ilimitados** | **≈1.300 op/s** | ESTIMADO: ≈103 merges Rescue en serie × 7,44 µs (§217) ≈ 0,77 ms por operación |
| objetivo RTGS que el repositorio fija | **≈21 op/s de media, 60-105 de pico** | ESTIMADO por el repositorio ([`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.2) |
| 10^6 op/s | ≥ ≈770 cadenas de raíces independientes; ≈4.000-4.900 núcleos solo para verificar; ≈0,82-0,87 millones de núcleos de cliente para generar | ESTIMADO: 10^6 / 1.300; 10^6 × 3,98-4,93 ms; 10^6 × 0,817-0,865 s |

El techo no es criptográfico. Es el **encadenamiento de raíces con un solo escritor**: un recibo
vale contra su raíz exacta o contra la raíz de arranque de su lote (§230, `spec/RPC.md` «Un lote por
raíz»). Pasar de una cadena a 770 deja de ser «un libro cerrado» y pasa a ser otro sistema.
«Millones por segundo» queda entre 9.500 y 47.600 veces por encima de lo que un RTGS pide
(10^6 / 105 y 10^6 / 21).

⚠️ **El dato que más importa no está en la pregunta: a pico RTGS, la vía suelta con disco va
justa.** ≈110-130 op/s frente a 60-105 dejan un margen de ≈1,04× a ≈2,2× (ESTIMADO, sin medir; el
RPC, los fsync y el flush salen de medidas en otra máquina, §229, §234 y §204). La vía de lote
(`zkssl_applyMany`, con la que el §230 midió 4,95 pagos/s, con un solo agregador) lo amplía, pero
tampoco tiene medida con disco después del §538. Por eso el primer corte después del 0 es **medir**.

**Qué optimizar**, en el orden que fijó el debate:

1. **Medir antes que tocar** (corte 1). Toda la línea base del nodo (2,49 ms de verificación,
   3,67 ms de apply, 248 op/s) es pre-§538. La sesión midió verificar y aplicar en la capa. Falta
   el nodo por RPC con `--ledger`, la capa a 10^5 cuentas y el reparto del probador con spans.
2. **Sacar del candado global el trabajo que crece con la historia** (REND-01, REND-02): las vistas
   de época y la cima del MMR se rehacen desde cero en cada latido y en cada `zkssl_epochHead`. No
   es throughput: es latencia de cola y un nodo que se degrada con la edad.
3. **Verificar en paralelo y aplicar en serie, sin clonar** (REND-03, REND-04), y **persistir una
   vez por lote** (REND-05). Tras el §538, verificar es ≈70-85 % del apply (ESTIMADO con las
   medidas de la sesión), no el 7 % con que el RFC-0002 descartó esta palanca.
4. **Cliente**: ocultar en la holgura de la traza en vez de duplicarla (ZK-3), recuperar los spans
   del probador (ZK-4), y todo lo que cambie los bytes de la prueba **en un único corte de cable
   zkssl/0.5**, después del juez único (ARQ-01).
5. **Memoria y arranque que crecen con la historia** (REND-07, REND-08, REND-09, REND-11, REND-12).
6. **Integración**: un juez único de verificación (ARQ-01), un nodo de producción que se pueda
   compilar (ARQ-02), errores tipados en el SDK (ARQ-03), un testigo sin el probador (ARQ-04) y el
   verificador como biblioteca (ARQ-06).

**Qué no optimizar**: perseguir 10^6/s; quitar o trocear el `Mutex` global (lo que serializa es la
raíz, §230); cambiar Rescue en los árboles de estado (lo sube el circuito); Pedersen, KZG, Groth16,
PLONK-KZG, Bulletproofs o «zero-sum proofs» (logaritmo discreto o ceremonia: chocan con la tesis
poscuántica); recursión de varios titulares; GPUs; fusionar los AIR v1 y v2. La tabla del §6 da la
razón y la fuente de cada descarte.

---

## 2. Línea base

### 2.1 Lo que midió la sesión

Máquina: contenedor en la nube, 4 vCPU, 15 GiB, rustc y cargo 1.97.0, release, `target` caliente
tras el canon. ⚠️ **Ruido declarado**: durante las medidas corrían en paralelo agentes de solo
lectura (load average 1,6-2,8). Todo sobre `7d13f26`; ningún fichero medido cambia hasta
`e69fadd`.

| fila | valor | fuente | etiqueta |
|---|---|---|---|
| canon `--sello` | **VERDE**, 16 min 45 s en frío (real), 52 min de CPU de usuario (≈54 con sistema) | `tools/canon.sh --sello` | MEDIDO |
| generar envío, 5 procesos | mediana **865,4 ms** (782,0-1.024,2) | `remedicion_89_1::muestra`, `crates/zk-ssl/src/metrics.rs:1046`; opciones de producción (`crates/zk-ssl/src/lib.rs:216-227`) | MEDIDO |
| generar cobro, 5 procesos | mediana **817,2 ms** (731,4-1.618,3) | ídem | MEDIDO |
| aplicar envío / cobro (capa en memoria: `new_layer`, sin sled) | mediana **5,5 / 5,1 ms** (4,8-6,2) | ídem | MEDIDO |
| prueba de envío / cobro | mediana **79.241 / 79.000 B** (78.505-79.301 y 78.231-79.928) | ídem | MEDIDO |
| verificar una prueba (media por proceso, envío y cobro) | **4,13 · 4,93 · 3,98 ms** → 242 · 203 · 251 verificaciones/s por núcleo | `el_coste_de_verificar_una_prueba`, `crates/zk-ssl/src/metrics.rs:247`, 3 procesos | MEDIDO |
| resto del apply (apply − verificar) | **0,8-1,8 ms** | ídem | MEDIDO |
| la verificación dentro del apply | **≈70-85 %** | 3,98-4,93 frente a 0,8-1,8 ms del resto | ESTIMADO |
| pago entero en serie, un hilo | **≈1,69 s** → ≈0,59 pagos/s | suma de las medianas: 865 + 5 + 817 + 5 ms | ESTIMADO |
| `medicion_130` con opciones de TEST | prove 592-649 ms; prueba 50,6-51,3 KB | `crates/stark-experiment/src/circuit_send.rs:1459` (32 consultas, blowup 8, sin molienda ni extensión) | MEDIDO, ⚠️ **no comparable**: otras opciones y sin la ocultación del §538 |
| forma de la traza de envío | 1024 filas × 58 columnas; oculta, 2048 × 59 | `crates/stark-experiment/src/circuit_send.rs:105`, `:147`; `comprobar_forma` (`crates/zk-ssl/src/lib.rs:241`) | MEDIDO |

### 2.2 Lo que el repositorio ya había medido

| fila | valor | fuente | etiqueta |
|---|---|---|---|
| banda publicada de un pago (envío + cobro) | **145.953-167.967 B** | `PUBLICADA_PAGO_MIN_B` y `PUBLICADA_PAGO_MAX_B`, `crates/zk-ssl/src/metrics.rs:82-83` (SALIDA-538M) | MEDIDO |
| generar con ocultación (mínimos) | envío 697,9-741,6 ms; cobro 696,9-715,6 ms | `crates/zk-ssl/src/metrics.rs:135` (§538) | MEDIDO |
| generar justo antes del §538 (apareado) | envío 158,9 ms; cobro 278,4 ms | [`AUDITORIA.md`](../AUDITORIA.md) §512 | MEDIDO, pre-§538 |
| coste de ocultar duplicando la traza (spike) | ×1,22 en bytes; ×3,2-3,3 al probar; ×1,9-2,0 al verificar | `spec/rfc/0009-lo-que-revela-una-prueba.md:314-315` (D-J) | MEDIDO |
| verificación sola (sin ocultar) | 2,49 ms envío, 2,43 ms cobro, sobre un apply de 33,8 ms | §204 A.2 | MEDIDO, pre-§538 |
| apply en la capa a 10^5 cuentas | 3,67 ms → 273 op/s; plano con las cuentas (e = 0,01) | §219 | MEDIDO, pre-§538 |
| banda de dispersión del apply suelto | 3,1-3,8 ms | §217 B.3 | MEDIDO, pre-§538 |
| techo del nodo por RPC (en memoria, 30 cuentas, i5-1135G7) | 0,225 + 4,035·n ms → **248 op/s** | §229 | MEDIDO, pre-§538 |
| merge Rescue «implícito» (con el `HashMap`) | 7,44 µs (`set_leaf`); 7,53 (§204); 8,91 (arranque) | §217 B.2 y B.4 | MEDIDO |
| fsync en ext4 · flush de sled | 0,907 ms · 0,55 ms | §234 · §204 | MEDIDO |
| RSS de la capa | 36 MB a 10^4 cuentas; 163 MB a 10^5 | §219 | MEDIDO |
| arranque a 10^5 cuentas con `rebuild_from` | 15,70 s (antes, 29,68 s) | §221 | MEDIDO |
| contención con 4 agregadores | aplica 1 de cada 4 por ronda; un rechazo cuesta 3,1 ms | §230 | MEDIDO |
| ciclo de pago, 4 titulares en un portátil | 4,95 pagos/s; el nodo es el 4 %, generar el 96 % | §222, §229 | MEDIDO, pre-§538 |
| firma XMSS de una cabeza | 18.519 B | `crates/zk-ssl-node/src/firma_cabeza.rs:323` | MEDIDO |
| verificaciones vivas en el árbol | 21 `verify::<…>` (15 en la capa, 10 de ellas en `two_phase.rs`; la pareja umbral; 5 en el kit) | §578 | MEDIDO |
| `--completo` del canon | 3.481 s, de los que 3.219 s (92 %) son halo2, plonk y zk-core | §629 | MEDIDO |
| paquetes del lock que solo existen por los comparativos | 104 de 329 nombres en `7d13f26`; 102 de 335 en `e69fadd`; curvas incluidas | script sobre `Cargo.lock`, clausura por nombre (agente de arquitectura, verificado; rehecho en `e69fadd` con el mismo script) | MEDIDO; los 6 nombres nuevos (11 paquetes) los trae `zk-ssl-medio` con `ml-dsa` (§632) |
| objetivo RTGS en operaciones de capa | ≈21 op/s de media; 60-105 de pico | [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.2 | ESTIMADO |

### 2.3 Cifras publicadas que han envejecido

Ninguna de estas cifras está mal como registro de su fecha. Lo que ha caducado es su uso como base
vigente.

| dónde | qué dice | qué hay hoy | etiqueta |
|---|---|---|---|
| [`ARQUITECTURA.md`](../ARQUITECTURA.md), tabla «Métricas de la capa, medidas» (línea 620) | transferencia: generar ~620 ms, aplicar ~4 ms, prueba **61.966 B** | envío: **865 ms**, **5,5 ms** y **79.241 B** (medianas de la sesión). La ocultación del §538 subió la prueba; las filas de emisión, destrucción y auditoría no se han vuelto a medir | MEDIDO (sesión) frente a pre-§538 |
| `spec/rfc/0002-lotes-y-transicion-de-hoja.md:311` | descarta la verificación en paralelo porque «la verificación es el 7 %» | ≈68 % combinando §204 A.2 (2,49 ms de verificación) y §219 (3,67 ms de apply), y ≈70-85 % tras el §538 | ESTIMADO |
| `spec/rfc/0002-lotes-y-transicion-de-hoja.md:308` y [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.4.ter, nivel 1 | descartan el group commit: la persistencia era el 3 % («1,26 ms de 38» en RFC-0002:308; «0,55 ms de 38» en DIAGNOSTICO §6.4.ter) | ≈2,36 ms de fsync y flush por operación suelta sobre ≈7,2-8,6 ms de la capa con disco (≈27-33 %, con el apply de la sesión) | ESTIMADO |
| [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §0.bis | apply de 37,99 ms → 26,3 op/s | superado por los 248 op/s del §229, y el propio §238 lo dice | MEDIDO, pre-§538 |
| `crates/zk-ssl/src/lib.rs:212-215` | 128 bits demostrables «costarían 125,6 KB en vez de 36,7» | cifras del circuito de cumplimiento (T = 512): 36,7 KB con q = 32, blowup 16 y sin molienda; 125,6 KB con q = 120, molienda 20 y extensión cúbica (`FIVE_BACKENDS.md` §4). No son las opciones de producción (ZK-2) | ESTIMADO |
| `PRINCIPIOS.md:324-325`, `doc/INSTITUCIONAL.md:450-457`, `FIVE_BACKENDS.md` §4 | «127 conjeturados conviven con 29-63 demostrables» | para las opciones 42/16/21 sobre la traza oculta: conjeturada 127, LDR 80 y UDR 59. Lo dio primero una réplica de `security.rs` (ZK-2), y lo confirma la función real con el boceto 4, corrido en una copia desechable de `e69fadd` | MEDIDO (sesión, boceto 4) |
| `doc/INSTITUCIONAL.md:445` | la degradación de Grover «está contemplada en la elección de parámetros» | sin fuente en el árbol (ZK-2) | ⚠️ sin medida |
| `crates/zk-ssl/src/metrics.rs:140-142` | envío 260,7-286,8 ms y cobro 159,4-189,6 ms | el propio árbol retiró esa compuerta en el S362 (`metrics.rs:148-153`): el cronómetro sumaba el trabajo del cliente y el de la capa. La base apareada pre-§538 es la del §512 | MEDIDO, superado |
| `doc/ESCALADO.md:146-149` | 1,85 pagos/s en serie (553 ms por pago) | ≈0,59 pagos/s (≈1,69 s por pago con las medianas de la sesión). ⚠️ Ese documento es cuerpo verbatim con cabecera-mapa (§120): se corrige en la cabecera, no en el cuerpo | ESTIMADO |
| §229 y §204 | 248 op/s y 2,49 ms | siguen siendo el registro, pero son pre-§538; no hay medida del nodo por RPC después | MEDIDO, pre-§538 |

---

## 3. Fase 1 — Diagnóstico

Cada tabla da el hallazgo como quedó **después** del verificador, el debate y la réplica. Cuando el verificador dejó un
hallazgo MATIZADO, manda su corrección; cuando hubo réplica, manda la propuesta final. Ningún
hallazgo publicable salió REFUTADO.

### 3.1 Rendimiento y estructuras de datos

**Resumen del especialista, corregido.** El camino crítico de un pago en el nodo es un único
escritor: `dispatch` toma el `Mutex` global `estado` en todos los métodos
(`crates/zk-ssl-node/src/main.rs:1841`). Lo que impone el orden total no es el candado sino la raíz
(§230), así que quitar el candado no compra nada. Lo que sí sobra está dentro: las vistas de época
(acuses, recibos y la cima del MMR) se rehacen hoja a hoja bajo el candado; `apply_many` clona los
árboles enteros en cada lote; la validación del lote corre en serie aunque es pura; hay tres fsync
por operación suelta, dos de ellos de la recepción del RFC-0010, por diseño del §570
(`crates/zk-ssl-node/src/latido.rs:262-265`); y la memoria y el arranque crecen con la
**historia**, no con las cuentas: el registro de transiciones vive entero en RAM
(`crates/zk-ssl/src/log.rs:296-300`).

| id | título | prio. | verificador | propuesta final tras el debate | evidencia principal |
|---|---|---|---|---|---|
| REND-01 | Los árboles de época (acuses y recibos) se rehacen hoja a hoja, copiando el registro entero, bajo el candado global | P1 | MATIZADO | Acuses: dos `FronteraDensa` de profundidad 32 (la de la época que se firma y la abierta), con los vacíos de `SparseTree::with_depth(32)`; la vieja se retira en `conservar`, no al componer. Recibos: frontera alimentada solo tras `anotar()` correcto, con hoja vacía por cada rx saltado y el mismo `RxRepetido`; antes de firmar, y fuera del candado, se recalcula desde disco, tolerando una entrada final parcial de rx > R, y, si discrepa, no se firma (PARADA). Latido: de ≈5-7,7 s/min con el escritor parado a ≈0,5 ms (ESTIMADO) | `crates/zk-ssl-node/src/vista_acuses.rs:37-62`; `latido.rs:150-153` declara el coste «NO medido» |
| REND-02 | El nodo se degrada con la edad: la cima del MMR es O(t) y el diario se relee entero, bajo el candado | P1 | MATIZADO (la cima O(t) ya la declaraba §292; lo nuevo es pagarla bajo el candado en `zkssl_epochHead`) | Cimas incrementales en `zk-ssl-verify::mmr`, junto a `cima`, con test contra `mmr::cima` y `tools/segunda/nucleo.py`; `pareja_mmr` y `conservar` fallan cerrados ante un candado envenenado; índice del diario en memoria, actualizado solo en `conservar` tras el fsync, que distingue el diario ausente (génesis) del ilegible (no arranca); `BTreeMap<era, max_rx>` en vez de `read_dir`; la firma del diario sigue en hex (cambiarla va por RFC). La cima pasa de ≈11,7 s por llamada al año de servicio a ≤ 128 merges (ESTIMADO) | `crates/zk-ssl-verify/src/mmr.rs:63-80`; `crates/zk-ssl-node/src/main.rs:1901` |
| REND-03 | `apply_many` clona en cada lote los árboles enteros de cuentas y pendientes, y el clon no protege nada | P1 | MATIZADO (cifra: 30-70 ms por lote a 10^5 cuentas, ESTIMADO) | DEFENDIDA: borrar los dos clones; `validate_*` recibe `&SparseTree`; `debug_assert` de que las tres raíces no cambian entre los pasos 3 y 4; banco H.1 a 10^4 y 10^5 cuentas antes y después | `crates/zk-ssl/src/two_phase.rs:1488-1489` |
| REND-04 | Verificar en paralelo y aplicar en serie: el descarte del RFC-0002 se apoya en un 7 % caducado | P1 | MATIZADO (la parte en serie del lote son ≈103 merges, no 64; cota ≈1.300 op/s) | Validar el lote con `std::thread::scope`, veredicto del **menor** índice (el de `apply_many`: la primera que falla, `spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md:41`; su índice viaja como `operacion: j`, D-B), cualquier fallo de un hilo convertido en rechazo con su índice, y prueba de equivalencia serie = paralelo con fallos en posiciones aleatorias. Antes de corregir RFC-0002 y DIAGNOSTICO, medir con probador y verificador en máquinas distintas. ≈460-530 op/s en 4 núcleos, en memoria (ESTIMADO, método en el §1). La réplica sostiene que la mejora relativa por operación (de ≈6 a ≈2 ms) cae fuera de la banda de ruido del §217 B.3; lo que exige medida es la cifra absoluta | `crates/zk-ssl/src/two_phase.rs:1492-1534`; `spec/rfc/0002-lotes-y-transicion-de-hoja.md:311` |
| REND-05 | Tres fsync por operación suelta; el 248 op/s es anterior a dos de ellos; el group commit se descartó con un denominador que ya no existe | P1 | MATIZADO (los fsync de recepción bajo el candado son diseño del §570, no descuido) | Persistir una vez por lote (`commit_lote`: un `apply_batch` y un `flush`), con prueba de equivalencia byte a byte frente a N commits sueltos y prueba de caída entre mutar y hacer flush. Medir H.1 con `--ledger`. Con disco, tras el §538: ≈113-115 op/s sueltas; lote de 15, de ≈147 a ≈159 op/s con `commit_lote` (+8 %); ≈460 op/s en 4 núcleos con REND-03 y REND-04 (ESTIMADO, réplica de rendimiento, sin medir; antes del §538 se estimaron ≈156 y ≈230). Reservar rx en bloques se retiró en la réplica (§6) | `crates/zk-ssl/src/persistence.rs:913-915`; `two_phase.rs:1443-1447` |
| REND-06 | Cada operación recalcula dos veces los mismos caminos (`root_with` al validar, `set_leaf` al aplicar) | P3 | MATIZADO (en lote no ahorra nada una vez aplicado REND-04) | Solo en la vía suelta: `raiz_y_camino_con` y `fijar_con_camino(…, raíz_base) -> Result`, que falla si `root()` no es la raíz base; tras fijar, igualdad con `pi.root_new`. ≈8-10 % por operación suelta (ESTIMADO: 0,48 ms, 64 × 7,44 µs, sobre el apply de 4,8-6,2 ms de la sesión) | `two_phase.rs:1050-1051` frente a `:1075-1076` |
| REND-07 | El registro de transiciones vive entero en RAM, y arrancar y servir consumos lo recorren en O(L) | P1 | CONFIRMADO | Por RFC y con migración, en dos pasos. (1) Cola en RAM y cuerpo en sled (claves `log:` en big-endian); al arrancar, `verify_chain` y todas las derivaciones recorren el registro en streaming desde disco: la RAM baja a O(época) sin cambiar ninguna garantía. Los contadores custodiados **siguen derivados** del registro (no de `meta:cust_uses`, por §393 y §394). Caminos de consumo históricos con un árbol versionado, no con una raíz por cabeza. (2) Después y aparte: verificación por tramos solo desde una cabeza XMSS cofirmada, con el prefijo verificado en segundo plano; solo este paso baja el arranque. ≈77 GB por año hábil de RAM a volumen Fedwire hoy (≈112 GB por año natural; ESTIMADO) | `crates/zk-ssl/src/log.rs:296-300`; `persistence.rs:590` |
| REND-08 | Pendientes: asignación por barrido lineal y un árbol denso guardado como `HashMap` disperso, clonado en cada latido | P2 | CONFIRMADO | `BTreeSet<u64>` de libres que excluye `reserved_pending`, reconstruido en `load`, con prueba de propiedad contra el barrido; representación densa por niveles solo para pendientes y meta, con prueba diferencial de raíz y `path_for` | `crates/zk-ssl/src/two_phase.rs:285-314`; `foto_pendientes.rs:82-88` |
| REND-09 | Restaurar una instantánea y cargar consumos siguen insertando hoja a hoja | P2 | CONFIRMADO | Rechazar índices repetidos o ≥ 2^depth y exigir biyección records↔hojas **antes** de `rebuild_from`, en `import_snapshot` y en `load`; después, `rebuild_from`. ~1,9× a 10^5 y ~2,6× a 10^6 (ESTIMADO, ley del §217) | `crates/zk-ssl/src/snapshot.rs:443`, `:485`, `:495`; `persistence.rs:566` |
| REND-10 | `dispatch` síncrono en los hilos de tokio, con el parseo dentro del candado global | P2 | CONFIRMADO | `preparar(method, params)` sin candado (parseo, conversión, `digest_of_proof`) y `ejecutar` con candado, en un hilo escritor dedicado o en `spawn_blocking`, de modo que los hilos de tokio dejan de bloquearse; la recepción de RFC-0010 sigue dentro del candado (§570); el digest calculado al preparar pasa a la capa (un productor). La firma XMSS del latido va en `spawn_blocking` y se espera antes del siguiente tic, con `GuardianIndice` bajo su `Mutex`. ~5 % de trabajo serie por op (0,216 de 4,035 ms, §229, sospecha declarada) | `crates/zk-ssl-node/src/main.rs:1454-1460`, `:1841` |
| REND-11 | 72,7 B por nodo interno: un mapa que a 10^6 cuentas ronda 880 MB | P3 | MATIZADO (el empaquetado propuesto no cabe en la profundidad 63 de consumos) | CONCEDIDA la objeción: mapa por nivel `Vec<HashMap<u64, Digest>>` con hasher **con clave** por proceso en cuentas, congeladas y consumos; pendientes y meta, densos (REND-08). −10-15 % de memoria del árbol (SUPUESTO) | `crates/zk-ssl/src/sparse_tree.rs:66-81` |
| REND-12 | `commit` reescribe en cada operación ~21 metadatos y el conjunto entero de congeladas | P3 | CONFIRMADO | Escribir `froz:` solo al congelar o descongelar, con su borrado (§390); los metadatos que cambian se derivan comparando con la última copia persistida, no con marcas a mano. Ganancia menor mientras mande el fsync (ESTIMADO) | `crates/zk-ssl/src/persistence.rs:772-819`, `:866-874` |

**Omisiones que el verificador añadió**, con su propuesta final: el barrido de reservas caducadas
es O(reservas) y no O(caducadas), como dice su comentario, y lo paga cada RPC bajo el candado
(`crates/zk-ssl-node/src/main.rs:1852-1855`; P2: una `VecDeque<(Instant, pos)>` con borrado
perezoso); el probador corre en un hilo porque nadie activa `concurrent` (ver ZK-4);
`digest_of_proof` se calcula dos veces por operación (se cierra con REND-10, con un único
constructor de la huella, que la capa exige); no hay perfil de release afinado (P3, por medir con el método del §217, y si se adopta el hash del kit se recalcula
en el mismo asiento); y los bancos no separan núcleos físicos de hilos y comparten máquina con el
probador (§229), así que sin un banco en máquinas distintas toda ganancia de REND-04/05/06 queda
dentro del ruido.

### 3.2 Criptografía y ZK

**Resumen del especialista, corregido.** La vía de producción usa q = 42, blowup 16,
molienda 21, extensión cuadrática, plegado FRI 8 y resto de grado 31
(`crates/zk-ssl/src/lib.rs:216-227`), Blake3-256 en el vector commitment y en Fiat–Shamir, y
Rescue-Prime `Rp64_256` dentro del circuito. Eso da **127 bits conjeturados**, y el tope lo pone el
campo de 128 bits, no las consultas (la fase de consultas aporta 4·42 + 21 = 189). Los bits
demostrables reales son **≈80 en LDR y ≈59 en UDR** según una réplica de
`crates/winter-air/src/proof/security.rs` que reproduce las cuatro filas de `FIVE_BACKENDS.md` §4;
la función real, corrida en la sesión con el boceto 4 (§5.4) sobre una prueba de envío oculta, da
lo mismo: 127, 80 y 59 (MEDIDO). Los documentos siguen citando los «29-63» de otro circuito y
otras opciones. Con q = 34 se conservan y los 80 en LDR, y la prueba pesa ≈17 % menos (ESTIMADO). En tiempo manda la
ocultación por duplicación de la traza. Bastan 64 filas aleatorias, y los cuatro circuitos calientes
dejan 200-208 filas libres. La molienda de 21 bits no aporta nada a la conjeturada y sí ≈6 bits a
la LDR, y el probador corre en un hilo. Ninguna tabla de seguridad está atada por un test: solo un
módulo comparativo llama a `proven_security` (`crates/stark-experiment/src/compliance_real_proof.rs:110-113`).

| id | título | prio. | verificador | propuesta final tras el debate | evidencia principal |
|---|---|---|---|---|---|
| ZK-1 | Las 42 consultas no compran bits en extensión cuadrática, y ningún test ata el nivel declarado | P1 | CONFIRMADO (q = 34 es el mínimo exacto: con 33, la LDR queda en 79) | En dos tiempos. (1) Ya, sin cable: un test que genera una prueba de envío **oculta real** y exige `conjectured_security ≥ 127` y `proven_security().ldr_bits() ≥ 80` con la función real; ≈1,3-1,6 s (MEDIDO: 1,27-1,57 s en cuatro ejecuciones del boceto 4, release, 4 núcleos) y un pin del canon. (2) En el corte único zkssl/0.5, después de ARQ-01 y replicado en `tools/segunda` (`OPCIONES_KIT`): elegir (34,21), (36,17) o (blowup 8, q = 46) con dos medidas previas, la fracción de la verificación que depende de q y la decisión de ZK-7. Declarar la UDR nueva (52 o 49). El ahorro (envío de ≈79 a ≈66 KB, ESTIMADO) cae en el cable y en la evidencia, no en el estado del nodo | `crates/zk-ssl/src/lib.rs:216-227`; `crates/winter-air/src/proof/security.rs:31-47` |
| ZK-2 | Las cifras de seguridad demostrable y poscuántica que se publican no son las de producción | P1 | CONFIRMADO | Solo prosa más el test de ZK-1(1): sustituir los «29-63» y el «125,6 frente a 36,7 KB» por lo que mida `Proof::proven_security` sobre las pruebas ocultas de envío y cobro, atado por `check_publicadas`. Publicar el coste de los 128 bits demostrables (cúbica, q ≈ 55, ≈111 KB frente a 79, ESTIMADO) como opción de despliegue regulado. La frase de Grover pasa a ser un nivel cuántico ESTIMADO con su método (≈64 bits en los términos que limita el campo, ≈85 en colisiones de Blake3-256 por BHT) o se retira | `PRINCIPIOS.md:324-325`; `doc/INSTITUCIONAL.md:445`, `:450-457` |
| ZK-3 | Ocultar duplicando la traza (2T) multiplica el tiempo de probar; 64 filas aleatorias caben en la holgura que ya existe | P1 | MATIZADO (la base pre-§538 es la del §512; el reparto del probador no está medido) | Medir antes el reparto con spans (ZK-4). Variante en el núcleo: cada AIR declara la constante `FILAS_LIBRES` (0 por defecto), atada por un test a `TRACE_LENGTH − 1 − ROW_PENDING_ROOT`; el probador sobrescribe las h = 64 últimas filas reales y las exenciones suben a base + h. **La h la fija el AIR, nunca la prueba**: la marca nueva solo anuncia la forma. Censo de holgura de los 23 probadores antes de decidir si conviven dos caminos. Falsador por AIR con un testigo malo en las últimas h + 1 filas. Envío ≈167-183 ms y cobro ≈292-320 ms, frente a ≈700 (ESTIMADO sobre un factor SUPUESTO: base del §512, en otra máquina, × 1,05-1,15 sin derivar) | `crates/winter-air/src/air/oculta.rs:57-90`; `crates/stark-experiment/src/circuit_send.rs:102-104` |
| ZK-4 | El probador corre en un hilo y el reparto de su tiempo no está medido: la vía oculta perdió las trazas de `tracing` | P2 | MATIZADO (molienda ≲ 110 ms de media, no 100-170) | (1) Ya, sin bytes: devolver los spans a `generate_proof_oculto` y añadir los de filas aleatorias, sal y cociente. (2) `concurrent` solo como feature opt-in (`paralelo`) de `zk-ssl-sdk` y `zk-ssl-cli`, con una puerta nueva: el kit en su propio `CARGO_TARGET_DIR`, o su sha256 compilado aparte igual al compilado junto al cli. Ganancia ×2-3 en FFT y Merkle (SUPUESTO). (3) La molienda, en el corte zkssl/0.5 | `crates/winter-prover/src/lib.rs:702`; `crates/winter-prover/Cargo.toml:39-45` |
| ZK-5 | La sal de Merkle: una llamada a `OsRng` por hoja, y ≈6,6 KB por prueba | P2 | CONFIRMADO | CONCEDIDA la cifra: una sola extracción de `OsRng` por compromiso, de la que las sales se derivan por contador con el `Hasher` genérico, ahorra las ≈70.000 llamadas al sistema de cada prueba, ≈14-35 ms (ESTIMADO), no 25-45, y no necesita ninguna dependencia nueva; como la sal viaja en cada apertura, no cambia el formato ni `tools/segunda`. La **retirada** de la sal cambia el vector commitment y va al corte zkssl/0.5, condicionada al argumento escrito que pide RFC-0009 D-I. Su peso: 6.567 B por prueba, MEDIDO en el spike de D-I (SPIKE-B-P4 r1); con las opciones de producción, ≈5,3 KB, ESTIMADO con el modelo del especialista, que no está en el árbol | `crates/zk-ssl-air/src/sal.rs:177-181`; `spec/rfc/0009-lo-que-revela-una-prueba.md:296-306` |
| ZK-6 | Falta el argumento de simulación de la ocultación, y la suite E2 solo cuenta literales | P2 | MATIZADO (el repositorio no promete conocimiento cero: promete «cero literales» y declara lo que no promete, RFC-0009 D-A y `SECURITY.md:520-526`) | Trabajo de especificación: un argumento HVZK escrito para la composición exacta del fork, y declarar en D-B lo que revelan por diseño las entradas públicas y la forma. La medida estadística de los valores abiertos se retiró en la réplica, también como banco fuera del canon: dos mil pruebas de envío costarían ≈23-25 min (ESTIMADO) y tiene poco poder de falsación. La correlación del tamaño tras el §538 ya está medida (`BACKLOG.md`, entrada 114, §620-§621) | `spec/rfc/0009-lo-que-revela-una-prueba.md:69-99` |
| ZK-7 | Agregar varias operaciones en una prueba: solo encaja el cobro agregado de un mismo titular | P3 | MATIZADO (la LDR baja ≈4 bits cada vez que la traza se multiplica por 4) | Solo un spike medido, sin RFC hasta que haya demanda: N ∈ {4, 16}, potencias de dos con relleno; medir filas, bytes, LDR con la función real (76 y 72 estimados con q = 42) y ms de apply contando las N retiradas de pendientes. Se decide junto con ZK-1, porque empuja q hacia arriba | [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.4.ter (líneas 488-510) |
| ZK-8 | El verificador interpola 48 columnas periódicas de 1024 filas en cada llamada | P3 | MATIZADO (la caché tiene que ir en `Oculta`, que no delega los polinomios; ahorro 6-12 %, no 20-40 %) | Sin cable: delegar `get_periodic_column_polys` en `oculta.rs` y un `OnceLock` en los AIR calientes, con el test «polinomio en caché = interpolado»; comparar el sha256 del kit antes y después y, si cambia, versión nueva. ≈0,3-0,6 ms por verificación (ESTIMADO). El ciclo 8 de ARK, solo en zkssl/0.5 | `crates/winter-verifier/src/evaluator.rs:28-37`; `crates/winter-air/src/air/oculta.rs:137-138` |

⚠️ **La línea base que el verificador corrigió**: la verificación pre-§538 de un envío era
2,49 ms (MEDIDO, §204 A.2), no 1,1-1,6 ms, que es la de la auditoría; y con ocultación se estimó
en ≈4,7-5,0 ms (× 1,9-2,0 de D-J). La sesión lo midió después: **3,98-4,93 ms** (§2.1).

### 3.3 Seguridad

**Resumen del especialista, corregido.** La conservación de cada transición se demuestra en el
AIR, con rangos de 63 bits. En claves, el guardián del índice XMSS protege contra la caída del
proceso, pero no contra dos procesos que abran el mismo contador. El AIR sigue sin especificación formal (`SECURITY.md` §3.1). Las herramientas de la casa
cubren restricciones vacuas, ranuras pisadas y celdas sin dueño; no cubren la suficiencia, las
relaciones entre varias columnas (la clase del §487) ni la frontera entre la capa y el AIR. Que el
operador lo ve todo está declarado. La fuga por tamaño de prueba está superada por medida
(`BACKLOG.md`, entrada 114: con los importes barajados en 12 corridas, 192 muestras, p 0,64 y
0,65, §621). Y la hipótesis de «millones de pruebas por segundo» no toca nada de esto: verificar más
rápido no cambia ningún resultado de solidez.

| id | título | prio. | verificador | propuesta final tras el debate | evidencia principal |
|---|---|---|---|---|---|
| SEC-1 | El guardián del índice XMSS no excluye a un segundo proceso sobre el mismo fichero de contador | P2 | MATIZADO (la reutilización con la misma semilla y contadores distintos ya la ejercieron §331 y §599, y solo la detectan los testigos) | Un campo `_cerrojo: File` abierto y bloqueado con `try_lock` en `GuardianIndice::abrir`, vivo mientras viva la estructura; cubre a la vez firma de cabeza, recepción y testigo. `rust-version = "1.89"` en el `Cargo.toml` del guardián (el fork declara 1.87). Falsador: dos guardianes sobre la misma ruta, y el segundo falla. Se declara lo que no cubre: NFS, quien ignore un bloqueo de aviso, una copia en otra máquina y la misma semilla con otro contador | `crates/zk-ssl-guardian/src/lib.rs:411-414`, `:419-444`, `:495-507` |
| SEC-2 | Configuración sin techo canónico: con `max_supply` ≥ p, la contabilidad en u64 y el campo divergen (el §16.7 no es solo usabilidad) | P2 | CONFIRMADO (exige mala configuración y dos custodios; no se alcanza con los valores por defecto) | CONCEDIDO el alcance: un ayudante `canon63` sobre el `MAX_VALUE` que la capa ya importa (`crates/zk-ssl/src/lib.rs:169`), aplicado a la configuración (`max_supply` y `regulatory_limit` en `new` y `open`); aritmética comprobada en la emisión. Declarar que un libro con `max_supply` ≥ 2^63 deja de abrir | `crates/zk-ssl/src/lib.rs:1044-1078` y `crates/zk-ssl/src/persistence.rs:20`; `crates/zk-ssl/src/mint.rs:43-63`; [`AUDITORIA.md`](../AUDITORIA.md) §16.7 |
| SEC-3 | Cobertura de solidez: las herramientas ven el interior del AIR, no la frontera capa↔AIR ni las relaciones entre varias columnas | P2 | CONFIRMADO | Un censo de la frontera por familia, análogo al de celdas FV-1: cada familia declara como **dato** qué hace la capa con cada campo de sus entradas públicas, un único `comprobar_enunciado` recorre esa tabla dentro del juez de ARQ-01, y un falsador de capa (una macro de test que reutiliza recibos, sin pruebas STARK nuevas) la recorre fila a fila. Un campo nuevo sin fila no pasa el canon | `crates/stark-experiment/src/mutation.rs:36-44`; `doc/VERIFICACION_FORMAL.md:286-290` |

### 3.4 Arquitectura y API

**Resumen del especialista, corregido.** La vía de producción no tiene costura de motor: el único
trait de motor, `SettlementProver` (`crates/settlement-prover/src/lib.rs:64`), es del estudio
comparativo y no lo usa ningún crate de producción. El acoplamiento a winterfell se reparte en 21
`verify::<…>` vivos (§578) y 42 `impl Prover`, y se concentra en un objeto-dios, `SovereignLayer`,
con 84 `pub fn` en 19 bloques `impl` repartidos en 18 ficheros. La costura correcta no es un motor
universal (Goldilocks, Rescue, la codificación del digest y XMSS son núcleo congelado,
RFC-0005 D-A), sino **un juez por familia en la frontera bytes + entradas públicas**, dentro de
`zk-ssl-air`, que el kit ya compila sin el probador (§463). Para integrar un sistema contable, el
mayor obstáculo no es criptográfico: no hay un nodo de producción compilable, el SDK aplana los
rechazos a texto, el cable arrastra la capa y el probador, y el verificador solo se usa como proceso.
Los comparativos cuestan el 92 % del `--completo` (§629). Las defensas medidas (los `=`, las puertas
de clausura, el `[patch]` por nombre, las filas del canon, v1 y v2 en ficheros separados) no se
tocan.

| id | título | prio. | verificador | propuesta final tras el debate | evidencia principal |
|---|---|---|---|---|---|
| ARQ-01 | No hay costura de motor: 21 `verify::<…>` y 42 `impl Prover` fijan winterfell a mano | P1 | MATIZADO (10 `verify` vivos en `two_phase.rs`, no 11; la puerta no puede ser «un grep global = 0») | ENMIENDA: `juzgar::<J: Juez>` en `zk-ssl-air/src/juez.rs`. **La política de aceptación es del juez, no de la familia**: un solo `OptionSet` (`zk_ssl_air::opciones()`), la forma oculta derivada por el juez y comprobada antes de construir el AIR, VC y moneda fijos. El trait solo aporta el AIR, sus dimensiones y su enunciado. Un motor o una versión nueva entra como otra (familia, versión) que elige el cable o el sobre. `zk_ssl::proof_options` delega en `zk_ssl_air::opciones` y `prove_send`/`prove_claim` pierden el parámetro `options`. Puerta: ningún `verify::<` vivo fuera de `juez.rs` en `zk-ssl/src` (tests, `metrics.rs` e `instrumento_*` aparte) ni en `zk-ssl-air/src` | `crates/zk-ssl/src/two_phase.rs:980-1023`; `crates/zk-ssl-air/src/lib.rs:593-617` |
| ARQ-02 | No hay un nodo de producción: sin `dev` no arranca y ese build no se compila nunca | P1 | CONFIRMADO | CONCEDIDA: flags `--custodian-root` y `--governance-root`, que rechazan el digest cero y quedan registrados al arrancar; `default = []` y `dev` opt-in, con un test que exija que `tests_support` no exista en el build de producción; una línea del canon con `--no-default-features`, más un test que arranque con las raíces pasadas por argumento. Los métodos de emisión, reembolso y quema por RPC van **después**, cada uno con su RFC, sus vectores y su falsador por RPC; `applyMintDelegated` recibe pruebas umbral generadas fuera del nodo | `crates/zk-ssl-node/Cargo.toml:21-22`; `crates/zk-ssl-node/src/main.rs:1425-1433` |
| ARQ-03 | El SDK convierte todo rechazo en texto | P1 | MATIZADO (los campos siguen dentro de la cadena; se pierde el tipo, no el dato) | `ErrorNodo` tipado y `Rpc::call_tipado`. La recepción del rechazo viaja como `Constancia` cruda (el tipo del §606), documentada como **afirmación del nodo**: no es evidencia hasta que el kit la juzga contra la cabeza firmada. El SDK no la verifica ni la reescribe (la réplica DEFIENDE esto en parte: verificarla en el SDK duplicaría lógica del kit y metería `zk-ssl-verify` en su clausura), no reintenta ni regenera por su cuenta, y su documentación prohíbe regenerar tras un error de transporte sin consultar si la operación se aplicó. `es_estado_viejo()` no se publica sin la consulta de ARQ-07: regenerar tras un timeout sin consultar puede pagar dos veces. `anyhow` queda solo en los ejemplos | `crates/zk-ssl-sdk/src/lib.rs:59-82` |
| ARQ-04 | `zk-ssl-wire` arrastra la capa, sled y el probador a todo consumidor de los DTO, testigo incluido | P1 | MATIZADO (el coste ya estaba medido y aceptado en §312 y §543; lo nuevo es endurecer la regla) | Partir el cable en un crate ligero (serde, `zk-ssl-hash`, `zk-ssl-verify`) y mover las conversiones a `zk_ssl::cable`; el testigo, a su propio binario con una puerta de clausura sin `winter-prover`, `winterfell`, `zk-ssl`, `stark-experiment` ni `sled`, y con prueba de vida (`xmss`, `zk-ssl-verify`). Crates, no features (RFC-0011 D-A). Se declara como decisión nueva | `crates/zk-ssl-wire/Cargo.toml:20-28`; `crates/zk-ssl-cli/src/prenda.rs:293-338` |
| ARQ-05 | `SovereignLayer` es un objeto-dios, y el puente ISO entra por la vía con la clave en la capa | P2 | MATIZADO (18 ficheros; un `Almacen` en memoria «para tener dos llamadores» es un llamador artificial: el §580 deja `podar` sin conectar hasta que haya un operador real) | Partir por rol sin cambiar semántica: los métodos que reciben clave (`send`, `claim`, `refund`, `deissue`) pasan a `tests_support`; el puente ISO se rehace sobre materiales. El trait `Almacen` solo con un integrador concreto y con su contrato escrito: lote atómico y durable al volver de `flush`; sellado por encima del trait; la puerta de integridad del arranque en la capa; un falsador con un almacén que falla a mitad de lote | `crates/zk-ssl/src/two_phase.rs:791`, `:866-867`; `crates/zk-ssl/src/iso.rs:425-487` |
| ARQ-06 | El verificador independiente solo se puede usar como proceso | P2 | MATIZADO (el bloqueo de wasm-bindgen es getrandom vía xmss, `doc/integracion-vertical-evaluacion.md:321`) | `zk-ssl-verify/src/sobre.rs` con `pub fn verificar_sobre(&Value) -> Veredicto` (sin `Default`, con `DeclaradaNoProbada` separado de `Verde` en el tipo); `main.rs` queda como envoltorio. Como biblioteca, `verificar_sobre` tiene que ser una función total: toda entrada, bien formada o no, da un `Veredicto`. En el mismo corte, un test que recorra los 316 vectores con una mutación por campo y exija un `Veredicto` Rojo. FFI y WASM solo con un llamador real | `crates/zk-ssl-verify/src/main.rs:1647-1659` |
| ARQ-07 | Integración con un libro mayor existente: sin consulta de idempotencia ni adaptador de identidades | P2 | MATIZADO (el cursor de eventos ya existe: `zkssl_logEntries {fromSeq, limit}`, `spec/RPC.md:88`) | Se quita la propuesta del cursor. Queda `zkssl_estadoDePrueba` (índice proofDigest → logSeq, O(1) frente a recorrer `logEntries` desde un seq), que devuelve solo aplicada o no y el logSeq, más la deduplicación ISO por (MsgId, EndToEndId) en el adaptador, no en la capa | `spec/RPC.md:88`, `:101`; `crates/zk-ssl/src/iso.rs:284-306` |
| ARQ-08 | Lo comparativo vive en el workspace de producción: 104 de 329 nombres del lock en `7d13f26` (102 de 335 en `e69fadd`), todas las curvas y el 92 % del `--completo` | P1 / P2 / P3 | MATIZADO (`sled` no sale de la capa; el precedente de MTC favorece extraer, no un segundo workspace) | P1, ya: un test de clausura sobre `Cargo.lock` por cada raíz de producción que prohíba `ark-*`, `halo2*`, `pasta_curves`, `dusk-*`, `nova-snark`, `pairing`, `group`, `ff`, `bls12_381`, `blst`, `*25519*`, `k256` y `p256`, con prueba de vida. P2: los 11 módulos sin llamador de `stark-experiment` se mudan a los comparativos, sin borrar nada (BACKLOG 56). P3, opcional: extraer o segundo workspace, declarando que se aparta de `doc/integracion-vertical-evaluacion.md` §5.5 y de `doc/MTC.md` | `Cargo.toml` (miembros); `crates/stark-experiment/src/lib.rs:1-8` |
| ARQ-09 | Ficheros monolito: qué mezclan y cómo partirlos sin reabrir la decisión del §598 | P2 | MATIZADO (la decisión es la del §598, RFC-0011, no la del §589) | Después del corte 0 y en sellos de mudanza pura (mover sin editar): `main.rs` del nodo en módulos del mismo binario (`args`, `app`, `arranque`, `modos`, `rpc/…`), sin biblioteca del nodo; `two_phase.rs` en pendientes, envío, cobro, lote, caducidad y emisión pendiente; `witness.rs` coordinado con ARQ-04; `crates/zk-ssl/src/tests.rs` (3.353 líneas) en submódulos por dominio. El enrutador recibe parámetros ya deserializados fuera del candado (REND-10) | `crates/zk-ssl-node/src/main.rs:1813`; [`AUDITORIA.md`](../AUDITORIA.md) §598 |
| ARQ-10 | Duplicación de circuitos: qué es vivo, qué es comparativo y qué **no** se fusiona | P3 | MATIZADO (las líneas compartidas, con `comm -12` sobre `sort -u`: 951 en envío, 896 en cobro y 204 en reembolso) | No fusionar v1 y v2 (identidad byte a byte del 0.2 y guardián de layout por fichero). Mover `circuit_mint_pending` y `circuit_recovery` a los comparativos tras mudar `ThresholdAuth`. Una macro de configuración del probador solo sin parámetros de hash, VC, moneda ni ocultación (fijos, y la ocultación siempre encendida fuera de `cfg(test)`), comprobada con `cargo expand` y con una prueba por probador verificada por el juez de ARQ-01 | `crates/stark-experiment/src/circuit_send_v2.rs:1-35` |
| ARQ-11 | Dependencias, fork y canon: qué se simplifica y qué es una defensa que no se toca | P1 / P3 | MATIZADO (`anyhow` se declara de una sola forma en producción) | P1, en un sello: `winterfell = "=0.13.1"` en los 6 crates (si un `cargo update` dejara de aplicar el `[patch]`, se compilaría el verificador original sin la ocultación); el lector de miembros de `canon.sh` estrechado a `members = [...]`, con falsador; una puerta común que exija que `winter-air`, `winter-prover` y `winter-verifier` no tengan `source` en la clausura de **toda** raíz de producción. P3: los pines a `[workspace.dependencies]`, sin dependencias por path allí | `tools/canon.sh:150-153`; `Cargo.toml:47` |

**Omisiones que el verificador añadió**: la línea base de rendimiento del especialista estaba
caducada (26,3 op/s frente a los 248 del §229); `zk-ssl` depende de `sled` directamente
(`crates/zk-ssl/Cargo.toml:28`), así que ninguna mudanza de `stark-experiment` lo saca de la capa; el
precedente de MTC desaconseja un segundo workspace en el mismo repositorio (`doc/MTC.md:12-15`); y
el coste del testigo que compila el probador ya está medido y aceptado en el §312.

---

## 4. Fase 2 — Debate

Cinco debates cruzados y cuatro réplicas. Cada tabla recoge solo las objeciones que **cambiaron**
algo, y cómo quedó en la réplica: CONCEDE (el autor acepta la objeción), DEFIENDE (mantiene su
propuesta) o ENMIENDA (la reescribe con la condición).

### 4.1 D1 — Seguridad cuestiona a Rendimiento

| hallazgo | objeción que cambió algo | réplica | qué quedó |
|---|---|---|---|
| REND-01 | Una frontera de recibos alimentada desde memoria firmaría lo que el proceso cree haber anotado, no lo que el disco tiene, y perdería la detección de `RxRepetido` (`crates/zk-ssl-node/src/vista_recibos.rs:25-31`); los rx tienen huecos (un número quemado es el caso seguro, `recepcion.rs:96-101`) | ENMIENDA | Frontera solo tras `anotar()` correcto, huecos como hoja vacía, y contraste desde disco **antes de firmar**, fuera del candado |
| REND-02 | `pareja_mmr` devuelve una cima génesis si su candado está envenenado (`latido.rs:461-469`): falla abierto, y un acumulador incremental lo heredaría; el diario es evidencia, no caché | ENMIENDA | Cimas en `zk-ssl-verify`, fallo cerrado, índice del diario actualizado solo en `conservar`, firma en hex hasta un RFC |
| REND-03 | Ninguna: los planes son dueños de sus datos (`two_phase.rs:133-140`, `:183-194`) | DEFIENDE | Igual, con el `debug_assert` de raíces |
| REND-04 | Validación en paralelo con veredicto determinista: un `AtomicUsize` con el menor índice fallido, cualquier fallo de un hilo convertido en rechazo con su índice, y prueba serie = paralelo; las cifras de ganancia caen dentro del ruido del §217 B.3 | ENMIENDA (y DEFIENDE que la mejora relativa, ≈3×, cae fuera del ruido) | Las cuatro condiciones, y medir en máquinas separadas antes de tocar RFC-0002 |
| REND-05 | Reservar rx en bloques obligaría a tolerar huecos de K − 1 números en `reconciliar`, y una pérdida real de anotaciones se confundiría con números quemados: debilita la detección de censura de RFC-0010 | ENMIENDA | Group commit sí, con sus dos falsadores; rx en bloques, retirado en la réplica (CONCEDE) |
| REND-06 | Una caché de caminos aplicada contra otro estado inserta nodos incoherentes en silencio | ENMIENDA (P3) | `fijar_con_camino` lleva la raíz base y falla si no casa; solo en la vía suelta |
| REND-07 | Sustituir la derivación de los contadores custodiados por `meta:cust_uses` reabre la clase del §393 y §394 (rebobinar un contador en reposo revivía autorizaciones gastadas): **rechazada esa parte** | ENMIENDA | Los contadores siguen derivados; verificación por tramos solo desde una cabeza cofirmada |
| REND-08 | Si la lista de libres diverge de la ocupación real, dos envíos reciben la misma posición | ENMIENDA | Prueba de propiedad contra el barrido lineal; denso solo donde la capa asigna la posición |
| REND-09 | `rebuild_from` y una secuencia de `set_leaf` no son equivalentes si la entrada repite un índice (`sparse_tree.rs:139-146` frente a `:204-213`), y ninguno comprueba la capacidad | ENMIENDA | Rechazo de repetidos y fuera de rango antes de `rebuild_from`, en `import_snapshot` y en `load` |
| REND-10 | Una firma XMSS lanzada en `spawn_blocking` sin esperarla solaparía dos latidos en la reserva de índice | ENMIENDA | La firma se espera antes del siguiente tic, con `GuardianIndice` bajo su `Mutex` |
| REND-11 | Un hasher multiplicativo sin clave sobre posiciones que no asigna la capa degrada las búsquedas con el candado tomado; `RandomState` lleva clave precisamente por eso | CONCEDE | Hasher con clave por proceso donde la posición no la asigna la capa |
| REND-12 | Unas marcas de cambio manuales son un segundo productor: un campo que nadie marque no se persiste nunca | ENMIENDA | Los cambios se derivan comparando con la última copia persistida |

**Propuestas conjuntas publicables**: *fronteras incrementales con contraste en disco antes de
firmar* (bajo el candado solo se lee la frontera; fuera de él, y antes de firmar con XMSS, se
recalculan desde disco las raíces de la época; si discrepan, no se firma); y *hasher con clave y
representación densa solo donde la capa asigna la posición*.

### 4.2 D2 — Criptografía cuestiona a Rendimiento

**La corrección transversal**: todas las cifras de rendimiento del nodo eran pre-§538. Con el
×1,9-2,0 de D-J, el debate estimó la verificación oculta en ≈4,7-5,0 ms y la operación suelta en
≈150-165 op/s en memoria. **La sesión lo midió después: 3,98-4,93 ms por verificación y 4,8-6,2 ms
por apply** (§2.1). La estimación del debate aguanta.

| hallazgo | objeción que cambió algo | réplica | qué quedó |
|---|---|---|---|
| REND-01 | La frontera de acuses es exacta (la hoja depende solo de `(proof_digest, seq + 1, n)`, `crates/zk-ssl-hash/src/lib.rs:696-699`), pero `n` debe fijarse al crearla, y la prueba de equivalencia ha de cubrir también la segunda implementación | ENMIENDA | Prueba en tres bandas: frontera = `raiz_de_epoca` = `tools/segunda/nucleo.py` |
| REND-02 | `nucleo.py` implementa `cima(hojas, con_hoja=True)`: las cimas incrementales tienen que guardar las hojas ya con `mmr_hoja` | ENMIENDA | Prueba para t en 1..=2^12 contra las dos implementaciones |
| REND-04 | Volver a medir el coste de verificar con la ocultación encendida antes de reabrir RFC-0002; el mismo `OptionSet` en cada hilo, sin estado mutable compartido | ENMIENDA | La sesión ya midió la verificación; falta el banco del nodo (corte 1) |
| REND-06 | Con la ocultación la ganancia baja del 13 % al ≈7-8 %, y la comprobación de la raíz base tiene que fallar, no ser un `debug_assert` | ENMIENDA | P3, solo vía suelta |
| REND-07 | Una raíz de consumos por cabeza **no da caminos**: `cons_path` histórico exige el árbol de esa cabeza (`crates/zk-ssl/src/consumo.rs:88-97`) | ENMIENDA | Árbol de consumos versionado (copia por camino: 63 nodos por inserción) o diario de inserciones; si no, `cons_path` solo contra la última cabeza, declarado |
| REND-11 | RECHAZAR en su forma original, por la misma razón que D1 | CONCEDE | Ídem |
| REND-12 | El nodo abre sled sin cifrado en reposo (`crates/zk-ssl-node/src/main.rs:1435-1438`) | — | Ya estaba declarado (`SECURITY.md:388-392`); no es nuevo |

**Propuestas conjuntas publicables**: *recalibrar la línea base con la ocultación encendida antes de
decidir nada*; *una compuerta de tres implementaciones para toda estructura incremental que alimente
un digest de cabeza* (la estructura nueva, la función de referencia de `zk-ssl-verify` y
`tools/segunda/nucleo.py`, sin que los vectores cambien un byte); *un punto de control del registro
anclado en el `chain_digest` firmado* (comprobar la firma XMSS de la última cabeza retenida y que
`entries[S-1].chain` coincide con lo firmado es equivalente a `verify_chain` frente a quien manipula
el disco sin la clave, y no frente al operador, como tampoco lo es hoy); y *medir con spans, en
máquinas separadas, informando bandas y no puntos*.

### 4.3 D3 — Seguridad y Criptografía cuestionan a Arquitectura

| hallazgo | objeción que cambió algo | réplica | qué quedó |
|---|---|---|---|
| ARQ-01 | Si «forma» y «opciones» son métodos que cada familia implementa, el trait permite que cada familia acepte una política distinta; la semilla de Fiat–Shamir no lleva identificador de familia (`crates/winter-verifier/src/lib.rs:100-101`, `crates/winter-air/src/proof/context.rs:119-136`) | ENMIENDA | Política de aceptación constante del juez; la versión la elige el cable o el sobre; un test de seguridad por cada `OptionSet` aceptado; la puerta, acotada como dijo el verificador |
| ARQ-02 | Exponer emisión, reembolso y quema por RPC abre superficie que hoy está cerrada | CONCEDE | Puntos 1-3 ya; los métodos nuevos después, cada uno con RFC, vectores y falsador |
| ARQ-03 | `es_estado_viejo()` invita a regenerar y reintentar: si el primer intento se aplicó y se perdió la respuesta, se paga dos veces; y lo que viene en un rechazo es una afirmación de un nodo que puede mentir (RFC-0011) | ENMIENDA (y DEFIENDE en parte que el SDK no verifique) | Recepción como `Constancia` cruda y nombrada como afirmación; la juzga el kit. Sin `es_estado_viejo()` público hasta ARQ-07 |
| ARQ-04 | Partir el cable refuerza la regla de que quien juzga no compila a quien prueba | ENMIENDA | Puerta del testigo aislado con prueba de vida; decisión nueva, no incumplimiento |
| ARQ-05 | Un `Almacen` de Postgres o de un mainframe sin atomicidad entre registros y raíces, o que reciba los valores sin sellar, rompe las tres defensas de hoy | ENMIENDA | El contrato escrito y probado, y solo con un integrador real |
| ARQ-06 | Como biblioteca, `verificar_sobre` tiene que ser una función total: toda entrada, bien formada o no, da un `Veredicto`. Un proceso aparte no necesitaba ese contrato | ENMIENDA | `Veredicto` para toda entrada, comprobado sobre los 316 vectores con una mutación por campo |
| ARQ-07 | Con el cursor ya existente, el método nuevo es una optimización O(1), no una capacidad que falte | ENMIENDA | Solo `zkssl_estadoDePrueba` y la deduplicación ISO en el adaptador |
| ARQ-08 | La ausencia de curvas en la vía de producción se sostiene hoy por disciplina: un `use ark_*` compilaría | ENMIENDA | Puerta anti-curvas ya, por raíz de producción, aunque los comparativos sigan en el lock |
| ARQ-09 | Las mudanzas moverían líneas que citan los asientos y los RFC | ENMIENDA | Después del corte 0, en mudanza pura |
| ARQ-10 | Una macro con parámetros de hash, VC, moneda u ocultación fijaría en 42 sitios cualquier error | ENMIENDA | Macro sin esos parámetros |
| ARQ-11 | El caret de `winterfell` es un riesgo de degradación: si el `[patch]` dejara de aplicarse, solo la puerta del cli lo vería | ENMIENDA | Pin `=`, y la puerta de «sin `source`» para toda raíz de producción |

**Propuestas conjuntas publicables**: *separación de dominio por familia y por versión de motor
dentro del transcript* (hoy la separación entre familias depende de que el tipo de AIR esté fijado
en cada sitio de llamada; el formato nuevo que el corte zkssl/0.5 trae de todos modos puede llevar
un identificador de familia y de versión de motor que entre en el transcript y que el juez compare
con el suyo); y *una puerta de clausura de producción* (sin curvas en ninguna raíz; sin
probador en quien juzga; el fork obligatorio, cada regla con su prueba de vida), generalizando
`la_clausura_del_kit_no_lleva_el_probador` (`crates/zk-ssl-verify/src/lib.rs:1235`).

### 4.4 D4 — Rendimiento y Arquitectura cuestionan a Criptografía

| hallazgo | objeción que cambió algo | réplica | qué quedó |
|---|---|---|---|
| ZK-1 | Bajar q es un cambio de cable cuyo coste no se contó: las opciones tienen cuatro productores (`crates/zk-ssl/src/lib.rs:216-227`, `crates/zk-ssl-air/src/lib.rs:150-161`, cinco asserts que los atan y `OPCIONES_KIT` en `tools/segunda/stark.py:206`), una sola `opciones()` sirve a las cinco familias del kit y el precedente del §538 fue un ENSAYO rojo 23/37 antes del verde 37/37 en 1.598 s | ENMIENDA | (1) ya; (2) dentro del corte zkssl/0.5, después de ARQ-01 |
| ZK-3 | Introduce una segunda forma de ocultación en el núcleo que el kit compila; la segunda implementación da por hecho 2T y un único prefijo de marca (`tools/segunda/stark.py:208`) | ENMIENDA | La réplica encontró además que la h no puede viajar en la marca: la fija el AIR (`FILAS_LIBRES`), y así el kit y Python rechazan igual |
| ZK-4 | `concurrent` activa `math`, `crypto`, `fri` y `utils` del fork, que son las mismas crates del kit; con `resolver = "2"` (`Cargo.toml:47`) y `tools/banco_completo.sh:84`, que compila nodo, cli y kit juntos, la unificación de features metería rayon en el binario del kit, y la puerta del kit no lo ve | ENMIENDA | Feature opt-in y una puerta de binario del kit |
| ZK-5 | El ahorro es de ≈14-35 ms, no de 25-45: desaparecen las llamadas al sistema, no los hashes | CONCEDE | Cifra rebajada |
| ZK-6 | Un test estadístico de dos mil pruebas cuesta ≈23-25 min en un hilo, no «unos segundos» | CONCEDE | Retirada (test y banco) |
| ZK-7 | Por rendimiento no hace falta: el objetivo RTGS ya cabe; la parte en serie del apply escala con N | ENMIENDA | Solo un spike medido |
| ZK-8 | El kit no verifica `SendAir` ni `ClaimAir`: el beneficio es del nodo, y ahí es un 5-10 % del apply | ENMIENDA | Caché en el AIR caliente y delegación en `Oculta`; se mide en el banco B.3 |

**Propuestas conjuntas publicables**: *volver a medir la línea base del nodo tras el §538* (B.3 e
H.1 con `--ledger`, con spans y con el probador en otra máquina); *un único corte zkssl/0.5 para
todo lo que cambia los bytes de la prueba, precedido por el juez único*; *una puerta de binario del
kit frente a la unificación de features* (el sha256 del kit compilado solo por `tools/artefacto.sh`
igual al compilado junto a cli y nodo); *el paquete de verificación del nodo* (caché de periódicas,
verificación paralela en lote y, en zkssl/0.5, ocultación en la holgura: ≈530 op/s y ≈690 op/s en
4 núcleos, ESTIMADO); y *un instrumento de seguridad en el canon alimentado por la propia prueba*,
que es además el único llamador de producción que les quedaría a esas funciones cuando ARQ-08 saque
`compliance_real_proof`.

### 4.5 D5 — Arquitectura cuestiona a Seguridad

| hallazgo | objeción que cambió algo | réplica | qué quedó |
|---|---|---|---|
| SEC-1 | Bloquear dentro de `persistir` no sirve (ese `File` se cierra al salir); el bloqueo tiene que vivir en un campo; `File::try_lock` es de Rust 1.89 y sube la MSRV del nodo, el cli y el testigo | ENMIENDA | Campo `_cerrojo`, `rust-version` declarada y lo que no cubre, dicho |
| SEC-2 | Envolver todas las conversiones de la capa es superficie sin riesgo medido; `canon63` va en la configuración | CONCEDE | `canon63` en la configuración |
| SEC-3 | Un falsador «que alguien enumere» deja de ser de puntos solo si la lista de lo que cada familia ata es un dato del código, no un comentario | ENMIENDA | La tabla por familia dentro del juez de ARQ-01 |

**Propuesta conjunta publicable**: *la frontera capa↔AIR declarada por familia dentro del juez*.
Cada familia declara como tabla constante sus entradas públicas con su clase; un solo
`comprobar_enunciado` la recorre antes de verificar; el falsador de SEC-3 itera la misma tabla; y
un campo nuevo en `*PublicInputs` sin fila no pasa el canon. Así el censo de la frontera pasa a ser
exhaustivo por construcción, como el censo de celdas FV-1.

### 4.6 Lo que el debate fijó como orden

Cinco reglas, que el §5.3 aplica tal cual: (1) **el corte 0 va primero**, y ninguna mudanza de
código va antes; (2) **medir primero**: ninguna decisión de rendimiento se toma con cifras
pre-§538, y ninguna ganancia dentro de la banda de ruido reabre un descarte publicado; (3) **un solo
corte de cable, zkssl/0.5**, para todo lo que cambia los bytes de la prueba, y **después** del juez
único, para que el cambio de `OptionSet` toque un sitio y no 21 (15 de la capa, la pareja umbral y
5 del kit, §578); (4) **una versión nueva del kit por tanda**, no una por cambio (precedente: el
§442 de [`AUDITORIA.md`](../AUDITORIA.md), que sube el kit a 0.2.0 una sola vez con las formas del
consumo, §417-§422, y del conflicto, §427-§431); y (5) **las mudanzas puras, en sellos propios y al
final**, porque mueven las líneas que citan los asientos.

---

## 5. Fase 3 — El Blueprint v2.0

### 5.1 Matriz comparativa: actual frente a propuesta

⚠️ La columna «Propuesta» es entera ESTIMADO o SUPUESTO: nada de ella está construido. Las
estimaciones parten de medidas pre-§538 corregidas con el ×1,9-2,0 de D-J o de las medidas de la
sesión, y caen, muchas, dentro de la banda de ruido del §217 B.3 hasta que el corte 1 las mida.

| dimensión | actual | etiqueta y fuente | propuesta | etiqueta y fuente |
|---|---|---|---|---|
| **generar** un envío (un hilo) | 865 ms (mediana); mínimos 698-742 ms | MEDIDO: sesión; `metrics.rs:135` | ≈167-183 ms envío y ≈292-320 ms cobro con la ocultación en la holgura (ZK-3); ×2-3 en FFT y Merkle con `paralelo` (ZK-4); −14-35 ms de sal (ZK-5) | ESTIMADO sobre un factor SUPUESTO (base §512, en otra máquina, × 1,05-1,15 sin derivar; el reparto del probador no está medido), SUPUESTO para `paralelo`, ESTIMADO para la sal |
| **verificar** una prueba | 3,98-4,93 ms | MEDIDO: sesión | −0,3-0,6 ms con la caché de periódicas (ZK-8); ≈2,7 ms con la holgura (×1,1 en vez de ×1,9) | ESTIMADO (D4 sobre §204 A.2) |
| **aplicar** una operación suelta | 4,8-6,2 ms en memoria; +≈2,4 ms de fsync y flush con disco | MEDIDO: sesión; ESTIMADO el disco (§234, §204, en otra máquina) | −≈0,48 ms en la vía suelta (REND-06); ≈3,9 ms con la holgura | ESTIMADO |
| **aplicar** en lote, por operación | 4,035 ms por op por RPC | MEDIDO, pre-§538 (§229) | ≈1,9 ms (4 núcleos, sin clon, verificación paralela y caché de periódicas, en memoria); ≈1,45 ms con la holgura | ESTIMADO (D4) |
| **tamaño** de prueba | 79.241 / 79.000 B; pago 145.953-167.967 B | MEDIDO: sesión; `metrics.rs:82-83` | envío ≈65,9 KB con q = 34; ≈73,1 KB con la holgura; ≈56,6 KB con las dos y sin sal | ESTIMADO (modelo del especialista, calibrado contra `metrics.rs`; no está en el árbol; la sal le pesa ≈5,3 KB, menos que los 6.567 B medidos en el spike de RFC-0009 D-I) |
| **throughput** del nodo | 248 op/s (pre-§538); ≈145-185 en memoria y ≈110-130 con disco hoy, operación suelta por RPC | MEDIDO (§229); ESTIMADO (§1) | ≈460-530 op/s en lote con 4 núcleos, en memoria; ≈690 con la holgura; cota ≈1.300 por cadena de raíces | ESTIMADO (§1; sin medir, el corte 1 lo convierte en banda) |
| **memoria** | 163 MB a 10^5 cuentas; 0,7-1,0 GB a 10^6; registro ≈184 B/entrada en RAM → ≈77 GB por año hábil a volumen Fedwire (≈112 GB por año natural) | MEDIDO (§219); ESTIMADO (proyección y registro) | registro en O(época) (RFC); árbol −10-15 % (REND-11); pendientes densos (REND-08) | ESTIMADO; SUPUESTO para REND-11 |
| **arranque** | 15,70 s a 10^5 cuentas; `verify_chain` entero: ≈75 s por día de historia Fedwire | MEDIDO (§221); ESTIMADO | O(época) desde un punto de control firmado; `import_snapshot` ~1,9× más rápido a 10^5 | ESTIMADO |
| **crecimiento con la historia** | latido: ≈5-7,7 s/min con el escritor parado; `zkssl_epochHead`: ≈11,7 s por llamada tras un año; diario en hex ≈19,5 GB/año | ESTIMADO (REND-01, REND-02) | ≈0,5 ms por latido; ≤ 128 merges por cima; índice del diario en memoria | ESTIMADO |
| **clon** por lote | 30-70 ms a 10^5 cuentas | ESTIMADO (REND-03) | 0 | ESTIMADO |
| **seguridad conjeturada** | 127 bits, sin ningún test del árbol que lo ate | MEDIDO: la función real sobre una prueba de envío oculta, con el boceto 4 corrido en una copia desechable de `e69fadd` (la fórmula de `security.rs:31-47` daba lo mismo) | 127, atado por un test en el canon sobre una prueba oculta real | ESTIMADO: el valor ya se midió; lo que falta es el test del árbol que lo ate |
| **seguridad demostrable** | LDR 80, UDR 59; se publica «29-63» | MEDIDO (ídem, boceto 4; la réplica de `security.rs` daba lo mismo) | LDR 80 con q = 34 (UDR 52); opción regulada de 128 demostrables con ≈111 KB | ESTIMADO |
| **superficie de verificación** | 21 `verify::<…>` vivos | MEDIDO (§578) | 1 juez; puerta estructural | ESTIMADO (ARQ-01) |
| **superficie de dependencias** | 104 nombres del lock solo por los comparativos (curvas incluidas); el cable arrastra el probador (116 paquetes); el testigo vive en el cli (197). Sobre `7d13f26`, y las clausuras cuentan la raíz; en `e69fadd`, 102, 118 y 199 | MEDIDO (script del agente de arquitectura, clausura por nombre; rehecho en `e69fadd`) | puerta anti-curvas por raíz; cable ligero; testigo de ≈197 a ≈170 paquetes sin probador, contando la raíz | ESTIMADO |
| **huecos de cobertura** publicables | frontera capa↔AIR sin herramienta; FV-2 solo sobre `circuit_refund`; build de producción nunca compilado; guardián XMSS sin exclusión entre procesos; **ocho hallazgos en embargo** | MEDIDO por lectura (SEC-1, SEC-3, ARQ-02); `doc/VERIFICACION_FORMAL.md:286-290` | censo de la frontera por familia; línea `--no-default-features` en el canon; cerrojo del guardián; corte 0 | ESTIMADO (lo que cubre); ⚠️ FV-2 y FV-3 siguen abiertos |
| **integración** | rechazos como texto; sin consulta de idempotencia; verificador solo como proceso; nodo de producción no compilable | MEDIDO por lectura (ARQ-02, ARQ-03, ARQ-06, ARQ-07) | `ErrorNodo`; `zkssl_estadoDePrueba`; `verificar_sobre`; flags de raíces | ESTIMADO (≈80 líneas del SDK; ≈2.300 líneas movidas del kit) |

### 5.2 Diagrama de bloques

Quién compila qué, dónde vive el juez único y por dónde pasa un pago. `[N]` es nuevo, `[C]` cambia
y lo que no lleva marca queda como hoy.

```text
 CLIENTE (máquina del titular)                    NODO (operador, un solo escritor)
 ┌──────────────────────────────────┐             ┌────────────────────────────────────────────┐
 │ zk-ssl-sdk  [C] ErrorNodo tipado │  zk-ssl-    │ zk-ssl-node                                │
 │ zk-ssl-cli       call_tipado     │  wire [C]   │  rpc/<grupo>::atender [C] (ARQ-09)         │
 │   └─ prove_send/claim [C]        │  ligero:    │  preparar() SIN candado [N] (REND-10)      │
 │      sin parámetro `options`     │  serde +    │   · parseo, conversión, huella             │
 │   └─ stark-experiment (AIR)      │  hash +     │  ── Mutex `estado` ─────────────────────── │
 │      └─ winter-prover (fork)     │  verify     │  │ SovereignLayer [C] (por rol, ARQ-05)  │ │
 │         spans [C]  `paralelo`[N] │◄───────────►│  │  sin clon por lote [C] (REND-03)      │ │
 │         ocultación en holgura    │  JSON-RPC   │  │  validate_* en paralelo [C] (REND-04) │ │
 │         [C, zkssl/0.5]           │             │  │  └─► juez ──────────────────────┐     │ │
 └──────────────────────────────────┘             │  │  commit_lote: 1 flush [C]        │     │ │
                                                  │  │  fronteras de época [N] (REND-01)│     │ │
                                                  │  ───────────────────────────────────│───── │
                                                  │  latido: cimas MMR [N] (REND-02),   │      │
                                                  │   contraste en disco fuera del      │      │
                                                  │   candado antes de firmar [N]       │      │
                                                  │   firma_cabeza XMSS ─ guardián [C]  │      │
                                                  │   (cerrojo entre procesos, SEC-1)   │      │
                                                  └─────────────────────────────────────│──────┘
                                                        cabeza firmada │               │
                                                                       ▼               │
 JUEZ ÚNICO DE VERIFICACIÓN [N] (ARQ-01)         ┌──────────────────────────────────┐ │
 ┌───────────────────────────────────────────┐   │ TESTIGO [C] (ARQ-04)             │ │
 │ zk-ssl-air/src/juez.rs                    │◄──┼─ binario propio: sin probador,   │ │
 │  juzgar::<J: Juez>(bytes, pi)             │   │  sin zk-ssl, sin sled            │ │
 │  · forma oculta derivada por el juez      │   │  cofirma la cabeza               │ │
 │  · OptionSet único = opciones()           │   └──────────────────────────────────┘ │
 │  · VC MerkleConSal, moneda Blake3 fijos   │◄────────────────────────────────────────┘
 │  · comprobar_enunciado: tabla por familia │
 │    (SEC-3)                                │   ┌──────────────────────────────────┐
 │ compilado por: zk-ssl (vía directa [C]),  │◄──┤ KIT arqueo-verify [C]            │
 │ stark-experiment (marcadores J), el kit   │   │ zk-ssl-verify: verificar_sobre   │
 │ NUNCA compila winter-prover (§463)        │   │ (sobre.rs [N]) → Veredicto       │
 └───────────────────────────────────────────┘   │ main.rs = envoltorio             │
                                                 │ tools/segunda (Python): oráculo  │
                                                 └──────────────────────────────────┘
 PUERTAS [N]: clausura de producción por raíz (sin curvas; sin probador en quien juzga; fork sin
 `source`), binario del kit igual compilado solo o junto al cli, instrumento de seguridad
 (conjeturada ≥ 127, LDR ≥ 80), compuerta de tres implementaciones para todo digest de cabeza.
```

**El flujo de un pago, paso a paso**, marcando lo que cambia respecto de hoy:

1. **Materiales.** El titular pide `zkssl_sendMaterials`. El nodo reserva una posición de
   pendiente. **[C] REND-08**: la posición sale de una lista de libres (`pop_first`), la misma que
   da hoy el barrido lineal, así que los vectores no cambian. **[C] omisión del barrido**: caducar
   reservas cuesta O(caducadas).
2. **Prueba de envío, en el cliente.** `prove_send` genera la prueba oculta. **[C] ARQ-01**: sin
   parámetro `options`, porque la capa solo acepta uno. **[C] ZK-4**: spans por fase, y
   `paralelo` como opción del binario cliente. **[C] zkssl/0.5**: ocultación en la holgura, q y
   molienda nuevas.
3. **Envío al nodo**, suelto o en lote, por un cable ligero **[C] ARQ-04**. **[N] REND-10**:
   deserializar, convertir y calcular la huella de la prueba fuera del `Mutex` global.
4. **Bajo el candado del estado**: el juicio y la mutación. Este documento no cambia el contrato
   del RFC-0010.
5. **Juicio.** **[C] ARQ-01**: `juzgar::<EnvioV1>` (o `EnvioV2`) sustituye al bloque de
   `crates/zk-ssl/src/two_phase.rs:980-1023`. En lote, **[C] REND-03 + REND-04**: sin clon y en
   paralelo, con el veredicto del menor índice.
6. **Aplicar.** En lote, **[C] REND-05**: mutar en memoria y persistir una vez (`commit_lote`).
   **[N] REND-01**: la frontera de acuses se alimenta con la entrada nueva (≈4 merges, ≈30 µs,
   ESTIMADO); la de recibos, solo tras anotar.
7. **Respuesta.** Si aplica, `Constancia`. Si rechaza, **[C] ARQ-03**: `ErrorNodo::Rechazo` con la
   causa del §454 y la recepción como `Constancia` cruda: una afirmación del nodo, que el SDK no
   verifica ni reescribe y que juzga el kit contra la cabeza firmada.
8. **Cobro.** Pasos 1 a 7 con `claimMaterials`, `prove_claim`, `juzgar::<CobroV1>` y la retirada
   del pendiente.
9. **Latido.** Bajo el candado solo se leen las fronteras **[C] REND-01** y la cima incremental
   **[C] REND-02** (O(1)) y se fija el límite. Fuera del candado y **antes de firmar**, se
   recalculan desde disco las raíces de la época; si discrepan, no se firma y el nodo pasa a
   PARADA **[N]**. Firma XMSS con el guardián, con **[C] SEC-1** un cerrojo entre procesos. La
   frontera vieja se retira en `conservar`.
10. **Testigo y kit.** El testigo, ya **[C] ARQ-04** un binario sin el probador, cofirma. El kit
    verifica los sobres con **[C] ARQ-06** `verificar_sobre`, que pasa por el mismo juez.

### 5.3 Plan de refactorización por cortes

Los cortes van en este orden. Ninguno empieza sin que el anterior esté verde en el canon. Cada uno
es candidato a bloque, con su camino rojo (paso 3 de [`GENAI.md`](../GENAI.md)).

#### Corte 0 — Los ocho hallazgos en embargo

Cerrar los ocho hallazgos entregados al autor en privado, con los falsadores que los acompañan.
Este documento no los detalla. Todo lo que sigue parte de un árbol con el corte 0 cerrado, y alguna
cifra de este documento puede moverse con él.

#### Corte 1 — Medir antes de tocar

- **Qué cambia**: nada de producción. Se corren B.3 (la capa a 10^5 cuentas) y H.1
  (`zkssl_applyMany` por RPC con `--ledger` en ext4, con y sin `--clave`) sobre la cabeza del árbol,
  con el probador en otra máquina y con núcleos físicos declarados.
- **Qué nace**: los spans de `generate_proof_oculto` (`crates/winter-prover/src/lib.rs:702`), con
  fases de LDE, restricciones, Merkle con sal, DEEP/FRI, molienda, filas aleatorias, sal y
  cociente. Opcionalmente, la medida de `lto` y `codegen-units = 1` con el método del §217.
- **Mueve**: ni cable, ni kit, ni vectores, ni la segunda implementación. Los spans no cambian
  bytes.
- **Riesgo**: bajo. El riesgo real es el contrario, decidir sin esto: repetir el error del §204
  (una cifra caducada que decide) en la otra dirección.

#### Corte 2 — Instrumentos y puertas, sin cable

- **Qué cambia**: `winterfell = "=0.13.1"` en los 6 crates que llevan caret; el lector
  `miembros_del_workspace` de `tools/canon.sh:150-153`, estrechado a `members = [...]`;
  `GuardianIndice` gana `_cerrojo` (`crates/zk-ssl-guardian/src/lib.rs:411-444`) y
  `rust-version = "1.89"`; `canon63` en `SovereignLayer::new` y `open` (`crates/zk-ssl/src/lib.rs:1044-1078` y
  `crates/zk-ssl/src/persistence.rs:20`); prosa de lib.rs:212-215, `PRINCIPIOS.md:324-325`,
  `FIVE_BACKENDS.md` §4 e `doc/INSTITUCIONAL.md:445-457` con la cifra del instrumento.
- **Qué nace**: el instrumento de seguridad (§5.4, boceto 4) con su pin en el canon; la puerta de
  clausura de producción (sin curvas por raíz, fork sin `source`, prueba de vida), que comparte el
  caminante del lock con la del kit como ayudante de tests, sin crate nuevo (propuesta del
  enjambre); falsadores del miembro fantasma, de los dos guardianes sobre la misma ruta y del tope
  ≥ 2^63.
- **Mueve**: ni cable ni vectores. El kit no cambia de binario (las puertas son tests).
  `check_publicadas` pasa a atar las cifras de seguridad.
- **Riesgo**: bajo. El cerrojo sube la MSRV del nodo, el cli y el testigo, y un libro con
  `max_supply` ≥ 2^63 deja de abrir: las dos cosas se declaran.

#### Corte 3 — El juez único y la biblioteca de sobres: una versión nueva del kit

- **Qué cambia**: nace `crates/zk-ssl-air/src/juez.rs` (`Juez`, `juzgar`, `Rechazo`). Las cinco
  funciones `verificar` del kit (`crates/zk-ssl-air/src/lib.rs:593`, `banda.rs`,
  `cobro_pendiente.rs`, `pago_en_curso.rs`, `prenda.rs`) pasan a marcadores de unas 7 líneas. En
  `stark-experiment`, marcadores `EnvioV1`, `EnvioV2`, `CobroV1`, `CobroV2`, `Reembolso`… junto a
  su AIR, sin mover ningún AIR. En `zk-ssl`, los 15 sitios de verificación (10 en `two_phase.rs`,
  más `burn`, la auditoría de `lib.rs`, `freeze`, `recovery` y `mint`) llaman a `juzgar`, y
  `zk-ssl` gana la dependencia directa de `zk-ssl-air` (hoy le llega por `stark-experiment`; el
  lock gana una arista y ningún paquete). El testigo del §578 que cuenta esas llamadas leyendo el
  fuente (`crates/zk-ssl/src/lib.rs:1145-1160`, diez en `two_phase.rs` y quince en total) pasa a
  contar `zk-ssl-air/src/juez.rs`, cuya única llamada lleva `MerkleConSal<Blake3>`, y descuenta
  las vías que migran. `zk_ssl::proof_options` delega en `zk_ssl_air::opciones` y `prove_send` y
  `prove_claim` (`crates/zk-ssl/src/client.rs:339`, `:473`) pierden `options`.
  `verify_threshold_pair` delega en el juez. `crates/zk-ssl-verify/src/main.rs` (45 funciones
  privadas, 3.126 líneas) pasa a `sobre.rs` con `verificar_sobre` y `Veredicto`; `main.rs` queda
  como envoltorio. La caché de periódicas de ZK-8 (delegación en `oculta.rs` y `OnceLock` en los
  AIR calientes) entra aquí, porque también cambia el binario del kit.
- **Qué nace**: la puerta estructural acotada (ningún `verify::<` vivo fuera de `juez.rs`); el test
  de seguridad por cada `OptionSet` aceptado; la tabla de la frontera por familia de SEC-3 en
  `comprobar_enunciado`, con el falsador de capa que la recorre; la mutación por campo sobre todos los
  vectores de los manifiestos del kit (siempre un `Veredicto` Rojo); el test «polinomio en caché = interpolado».
- **Mueve**: **el kit** (hash del binario nuevo, una sola versión de `arqueo-verify` para todo el
  corte). Ni cable ni vectores: ningún byte de prueba cambia, y los vectores de los manifiestos, los falsadores de
  forma del §529 y `conformance --check` tienen que salir idénticos. La segunda implementación no
  cambia.
- **Riesgo**: medio. Toca las 21 rutas de verificación vivas, y un error cambia qué se acepta. La
  red es que nada de lo que hoy se acepta o se rechaza puede cambiar de lado.

#### Corte 4 — El escritor único, más corto (nodo y capa, sin cable)

- **Qué cambia**: en `apply_many_con_operacion` (`crates/zk-ssl/src/two_phase.rs:1464-1545`), sin
  clon (REND-03) y validación en paralelo (REND-04, boceto 2); `commit_send` y `commit_claim` se
  parten en `mutar_*` + `commit`, y nace `commit_lote` en `crates/zk-ssl/src/persistence.rs` junto
  al `commit` de hoy (REND-05). En el nodo, `FronteraDensa` (en
  `crates/zk-ssl/src/sparse_tree.rs`) y dos fronteras de acuses y una de recibos en el `Estado`;
  `vista_acuses::pares` deja de copiar el registro (REND-01). `CimasMmr` en
  `crates/zk-ssl-verify/src/mmr.rs`; `pareja_mmr` y `conservar`
  (`crates/zk-ssl-node/src/latido.rs:461-469` y `:425-451`) fallan cerrados; índice del diario en
  memoria; `BTreeMap<era, max_rx>` en `registro_recepcion.rs` (REND-02). Cola de caducidad de
  reservas (`crates/zk-ssl-node/src/main.rs:1852-1867`). REND-06, si se adopta, solo en la vía
  suelta.
- **Qué nace**: prueba serie = paralelo con fallos en posiciones aleatorias; equivalencia byte a
  byte entre N commits y `commit_lote` (un scan completo de sled), incluidos los borrados; caída
  simulada entre mutar y hacer flush; la compuerta de tres implementaciones (frontera =
  `raiz_de_epoca` = `nucleo.py`, con huecos de rx y con el árbol lleno a una profundidad pequeña;
  cimas = `mmr::cima` = `nucleo.cima`); el test «la
  cabeza del latido es la que sirve el RPC» también durante la ventana de firma; el banco H.1 del
  corte 1, antes y después.
- **Mueve**: ni cable, ni kit, ni vectores (las raíces son las mismas). La segunda implementación
  es oráculo, no consumidor.
- **Riesgo**: medio. La frontera vieja tiene que retirarse en `conservar` y no al componer, o el
  RPC diverge del latido durante los ≥144,5 ms de firma. `SovereignLayer` tiene que ser `Sync`.

#### Corte 5 — Memoria, arranque e historia

- **Qué cambia**: `allocate_pending` con lista de libres y representación densa de pendientes y
  meta (REND-08; `crates/zk-ssl/src/two_phase.rs:285-314`, `foto_pendientes.rs:82-88`);
  `import_snapshot` y el bloque de consumos de `load` con `rebuild_from`, tras rechazar repetidos y
  fuera de rango (REND-09; `snapshot.rs:443`, `:485`, `:495`; `persistence.rs:566`); mapa por nivel
  con hasher con clave en cuentas, congeladas y consumos (REND-11; `sparse_tree.rs:66-81`); `froz:`
  solo al congelar y descongelar, y metadatos por diferencia (REND-12; `persistence.rs:772-874`).
  Después, por RFC: el registro con cola en RAM y cuerpo en sled, claves `log:` en big-endian,
  punto de control anclado en el `chain_digest` firmado y árbol de consumos versionado (REND-07).
- **Qué nace**: pruebas de propiedad (lista de libres frente al barrido; denso frente a
  `SparseTree` en raíz y `path_for`); falsadores de instantánea con índice repetido y fuera de rango;
  prueba de profundidad 63 con la posición `u64::MAX >> 1`; el test de reinicio del §390; para
  REND-07, el arranque que verifica la firma de la última cabeza y el prefijo en segundo plano, con
  PARADA si falla.
- **Mueve**: REND-07 cambia el **formato en disco** (migración, RFC). Lo demás, nada visible: ni
  cable, ni kit, ni vectores.
- **Riesgo**: medio en REND-08 y REND-11 (otra implementación del árbol que sostiene todas las
  raíces) y en el paso (1) de REND-07, que conserva `verify_chain` entero; alto solo en su paso
  (2), la verificación por tramos, que choca en parte con «negarse a arrancar si el estado no
  cuadra» ([`ARQUITECTURA.md`](../ARQUITECTURA.md)) y solo es equivalente si el punto de control
  está atado a una cabeza firmada y cofirmada.

#### Corte 6 — Integración: nodo de producción, SDK y testigo

- **Qué cambia**: `Args` y `open_layer` del nodo con `--custodian-root` y `--governance-root`
  (`crates/zk-ssl-node/src/main.rs:1425-1433`), que rechazan el digest cero y quedan registrados
  al arrancar, `default = []` en `crates/zk-ssl-node/Cargo.toml` y los bancos de `tools/` con
  `--features dev` (ARQ-02, puntos 1-3). `ErrorNodo` y `call_tipado` en
  `crates/zk-ssl-sdk/src/lib.rs` (ARQ-03). `zkssl_estadoDePrueba`, aditivo (zkssl/0.4 no sube por
  métodos nuevos), con su vector (ARQ-07). El cable partido: `zk-ssl-wire` ligero y las
  conversiones a `zk_ssl::cable`; el testigo, a su binario (ARQ-04).
- **Qué nace**: la línea `cargo build --release -p zk-ssl-node --no-default-features` en el canon,
  con un test que exija que `tests_support` no exista en ese build y otro que arranque con las
  raíces pasadas por argumento; dos tests del SDK sin nodo, en el molde del §606; la puerta de
  clausura del testigo con prueba de vida; la fila del método nuevo en el canon.
- **Mueve**: el **cable** de forma aditiva (un método nuevo con su vector; los DTO no cambian de
  forma). El kit no cambia. La firma pública del SDK sí (versión 0.1.0, sin consumidores externos
  medidos).
- **Riesgo**: bajo para la solidez; de fricción, porque invertir el valor por defecto de `dev` rompe
  las órdenes de los bancos, que hay que editar en el mismo sello. Los métodos de emisión, reembolso
  y quema por RPC (ARQ-02, punto 4) no van aquí: llegan después, cada uno con su RFC.

#### Corte 7 — El único corte de cable: zkssl/0.5

- **Qué cambia**: las opciones de `crates/zk-ssl-air/src/lib.rs:150-161` (y, por delegación, las de
  `crates/zk-ssl/src/lib.rs:216`) al punto del frente que fijen las medidas del corte 1 (ZK-1); la
  molienda (ZK-4); la ocultación en la holgura con `FILAS_LIBRES` por AIR en
  `crates/winter-air/src/air/oculta.rs` y en el probador (ZK-3); la retirada de la sal, si el
  argumento de D-I está escrito y revisado (ZK-5); el ciclo 8 de ARK (ZK-8); y un identificador de
  familia y de versión de motor dentro del transcript, que el juez compara con el suyo (D3).
- **Qué nace**: vectores 0.5 (los 0.4 apartados, como el §538 apartó los 0.3); bandas nuevas de
  `PUBLICADA_PAGO_MIN_B` y `PUBLICADA_PAGO_MAX_B`; el instrumento de seguridad con el `OptionSet`
  nuevo (y con los dos, si conviven por versión del recibo con una ventana cerrada con fecha); un
  falsador por AIR con testigo malo en las últimas h + 1 filas; la réplica en `tools/segunda`
  (`OPCIONES_KIT`, `marca_de`, `verificar_con_sal`), si cambian las familias del kit, y cambian,
  porque comparten `opciones()`.
- **Mueve**: **todo**: cable, kit, vectores y segunda implementación. Es el corte caro, y por eso es
  uno solo.
- **Riesgo**: alto. El precedente medido es el §538: cuatro generadores, 79 POST y un ENSAYO rojo
  23/37 antes del verde 37/37 en 1.598 s. Partirlo en cuatro cuadruplica ese coste y obliga a
  convivir con cuatro conjuntos de opciones o de marcas.

#### Corte 8 — Mudanzas puras y comparativos

- **Qué cambia**: en sellos de mudanza pura, `crates/zk-ssl-node/src/main.rs` en módulos del mismo
  binario y `dispatch` (líneas 1813-3052) en un enrutador por grupos; `two_phase.rs` (con la
  emisión pendiente aparte), `witness.rs` y `crates/zk-ssl/src/tests.rs` partidos (ARQ-09).
  Después, y no en el mismo sello, el enrutador recibe parámetros deserializados por `preparar()`
  fuera del candado (REND-10). Los métodos con clave a `tests_support` y el puente ISO sobre
  materiales (ARQ-05). Los 11 módulos sin llamador de `stark-experiment` a los comparativos y la
  cabecera de `crates/stark-experiment/src/lib.rs:1-8` corregida (ARQ-08 P2). La macro de
  configuración del probador, si se adopta (ARQ-10).
- **Qué nace**: nada nuevo de solidez; `tools/verificar_citas.py` y `tools/check_constraint_layout.py`
  como red; el recuento del censo de AIR de RFC-0009 D-G.
- **Mueve**: el kit no; el cable no. Cambian rutas que citan los documentos.
- **Riesgo**: de solidez, bajo; de fricción, alto: los asientos y los RFC citan `fichero:línea` por
  todas partes. Por eso va al final y en sellos propios.

#### Fuera de los cortes: spikes y especificación

- **ZK-7**, el cobro agregado: un spike medido, sin RFC hasta que haya demanda.
- **ZK-4 (2)**, la feature `paralelo`: tras los spans del corte 1, en una máquina sin el nodo, y con
  la puerta de binario del kit.
- **ZK-5 (1)**, la sal derivada (una extracción de `OsRng` por compromiso): no cambia cable,
  formato, vectores ni `tools/segunda`; puede ir en cualquier corte sin cable, con el sha256 del
  kit comparado antes y después y, si cambia, dentro de la versión del corte 3.
- **ZK-6**, el argumento HVZK: trabajo de especificación. Es trabajo para un revisor criptográfico
  externo, junto con el resto del fork.

#### El plan, módulo a módulo

La misma información que los cortes, leída por crate. El corte 0 no figura: su alcance lo tiene el
autor.

| módulo | cortes | qué cambia |
|---|---|---|
| `zk-ssl` (la capa) | 2, 3, 4, 5, 6, 8 | `canon63` en `new` y `open` y el instrumento de seguridad (2); las 15 verificaciones llaman a `juzgar`, `proof_options` delega y `prove_send`/`prove_claim` pierden `options` (3); lote sin clon y en paralelo, `commit_lote` y `FronteraDensa` (4); lista de libres, `rebuild_from` al restaurar, mapa por nivel, `froz:` y, por RFC, el registro en sled (5); recibe las conversiones del cable en `zk_ssl::cable` (6); `two_phase.rs` y `tests.rs` partidos, los métodos con clave a `tests_support` y el puente ISO sobre materiales (8) |
| `zk-ssl-air` | 3, 7 | nace `juez.rs` y las cinco `verificar` del kit pasan a marcadores (3); las opciones del corte de cable, q y molienda (7) |
| `stark-experiment` | 3, 7, 8 | marcadores `Juez` junto a cada AIR, `OnceLock` de periódicas en los AIR calientes y `verify_threshold_pair` por el juez (3); `FILAS_LIBRES` por AIR (7); los 11 módulos sin llamador a los comparativos y, si se adopta, la macro del probador (8) |
| `winter-air` (fork) | 3, 7 | delegación de las periódicas en `oculta.rs` (3); ocultación en la holgura (7) |
| `winter-prover` (fork) | 1, 7 | spans de `generate_proof_oculto` (1); el probador de la holgura (7); la feature `paralelo`, fuera de los cortes |
| `zk-ssl-verify` (el kit) | 3, 4, 7 | `sobre.rs` con `verificar_sobre` y una versión nueva del kit (3); `CimasMmr` en `mmr.rs` (4); opciones y formato nuevos (7) |
| `zk-ssl-node` | 4, 6, 8 | fronteras de época, cima incremental, índice del diario y cola de caducidad (4); flags de raíces y `default = []` (6); `main.rs` en módulos y `preparar()` fuera del candado (8) |
| `zk-ssl-sdk` | 6 | `ErrorNodo` y `call_tipado` |
| `zk-ssl-wire` y `zk-ssl-cli` | 6 | cable ligero; el testigo sale del cli a un binario propio sin probador |
| `zk-ssl-guardian` | 2 | `_cerrojo` y `rust-version = "1.89"` |
| `Cargo.toml` de los crates | 2 | `winterfell = "=0.13.1"` donde hoy hay caret |
| `tools/` | 2, 6, 7 | `canon.sh` y `check_publicadas` (2); bancos con `--features dev` y la línea del canon (6); `tools/segunda` (7) |

### 5.4 Bocetos de código

Cinco bocetos de las piezas más críticas que se pueden publicar. Los nombres y las firmas que
**usan** son los del árbol, comprobados con `grep` y con `cargo check`; los que **crean** son
nuevos. Cada boceto se insertó en su sitio en una copia desechable de `e69fadd`, fuera del
repositorio, y se compiló allí: su «Estado» dice con qué orden y qué salió. Ninguno compila aislado,
porque son fragmentos. **Ningún boceto incluye las condiciones del corte 0, que el autor tiene;
ninguno es completo sin ellas.** El cerrojo de SEC-1 no lleva boceto: es un campo
`_cerrojo: File` y una llamada a `try_lock` en `GuardianIndice::abrir`.

#### Boceto 1 — El juez único (ARQ-01, D3)

Estado: **compila** en su sitio (`cargo check --release -p stark-experiment -p zk-ssl`, sin
avisos). Los dos primeros fragmentos compilaron tal cual en la revisión; el tercero no compilaba,
porque `zk-ssl` no depende de `zk-ssl-air` y faltaba `EnvioV2`, y con lo que el boceto lleva ahora
compila. Rompe, como se espera, el testigo del §578, que cuenta diez `verify::<` en `two_phase.rs`:
con el boceto quedan ocho (MEDIDO), y el corte 3 lo reescribe.

```rust
// crates/zk-ssl-air/src/juez.rs (nuevo). Sin winter-prover: la clausura del kit no cambia (§463).
use winter_air::{proof::Proof, Air};
use winter_crypto::DefaultRandomCoin;
use winter_math::fields::f64::BaseElement;
use winter_verifier::{verify, AcceptableOptions};
use crate::{sal::MerkleConSal, Blake3}; // `type Blake3 = Blake3_256<BaseElement>` (lib.rs:81)

/// Lo que una familia declara de su traza SIN ocultar. La forma oculta la deriva el juez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimensiones { pub ancho: usize, pub ancho_aux: usize, pub aleatorios_aux: usize, pub filas: usize }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rechazo {
    Enunciado(String),
    Malformada(String),
    Forma { real: (usize, usize, usize, usize), exige: (usize, usize, usize, usize) },
    Prueba { familia: &'static str, causa: String },
}

/// Una FAMILIA: su AIR, sus dimensiones y su enunciado. NO aporta la política de aceptación
/// (opciones, forma oculta, VC, moneda): esa es una constante del juez (D3, condición a). Una
/// versión nueva de motor o de familia es OTRO tipo, que elige el cable o el sobre.
pub trait Juez {
    const FAMILIA: &'static str;
    type Air: Air<BaseField = BaseElement>;
    fn dimensiones(pi: &<Self::Air as Air>::PublicInputs) -> Dimensiones;
    /// Recorre la tabla de la frontera de la familia (SEC-3).
    fn comprobar_enunciado(_pi: &<Self::Air as Air>::PublicInputs) -> Result<(), String> { Ok(()) }
}

/// Juzga una prueba de la familia J con la política constante de la casa.
pub fn juzgar<J: Juez>(bytes: &[u8], pi: &<J::Air as Air>::PublicInputs) -> Result<(), Rechazo>
where
    <J::Air as Air>::PublicInputs: Clone,
{
    J::comprobar_enunciado(pi).map_err(Rechazo::Enunciado)?;
    let proof = Proof::from_bytes(bytes).map_err(|e| Rechazo::Malformada(format!("{e:?}")))?;
    let i = proof.trace_info();
    let real = (i.main_trace_width(), i.aux_segment_width(), i.get_num_aux_segment_rand_elements(), i.length());
    let d = J::dimensiones(pi);
    // La forma oculta de RFC-0009 D-G (ancho + 1, 2T).
    let exige = (d.ancho + 1, d.ancho_aux, d.aleatorios_aux, 2 * d.filas);
    if real != exige { return Err(Rechazo::Forma { real, exige }); }
    // Política constante: un solo OptionSet, el de la casa (lib.rs:150).
    let aceptadas = AcceptableOptions::OptionSet(vec![crate::opciones()]);
    verify::<J::Air, Blake3, DefaultRandomCoin<Blake3>, MerkleConSal<Blake3>>(proof, pi.clone(), &aceptadas)
        .map_err(|e| Rechazo::Prueba { familia: J::FAMILIA, causa: format!("{e:?}") })
}

// crates/stark-experiment/src/juez.rs (nuevo). El AIR no se mueve, y stark-experiment ya
// depende de zk-ssl-air (crates/stark-experiment/Cargo.toml:21): no hay ciclo. Las dos familias
// de envío comparten SendPublicInputs (circuit_send_v2.rs:51).
use crate::circuit_send::{SendAir, SendPublicInputs, TRACE_LENGTH, TRACE_WIDTH};
use zk_ssl_air::juez::{Dimensiones, Juez};
pub struct EnvioV1;
impl Juez for EnvioV1 {
    const FAMILIA: &'static str = "envio/v1";
    type Air = SendAir;
    fn dimensiones(_: &SendPublicInputs) -> Dimensiones {
        Dimensiones { ancho: TRACE_WIDTH, ancho_aux: 0, aleatorios_aux: 0, filas: TRACE_LENGTH }
    }
}
pub struct EnvioV2;
impl Juez for EnvioV2 {
    const FAMILIA: &'static str = "envio/v2";
    type Air = crate::circuit_send_v2::SendV2Air;
    fn dimensiones(_: &SendPublicInputs) -> Dimensiones {
        use crate::circuit_send_v2 as v2;
        Dimensiones { ancho: v2::TRACE_WIDTH, ancho_aux: 0, aleatorios_aux: 0, filas: v2::TRACE_LENGTH }
    }
}

// crates/zk-ssl/src/two_phase.rs, validate_send: sustituye el bloque de las líneas 980-1023.
// Requiere `zk-ssl-air = { path = "../zk-ssl-air" }` en crates/zk-ssl/Cargo.toml (corte 3), y
// retira de two_phase.rs los imports que quedan sin uso, `SendAir` y `SendV2Air`.
match receipt.notice.x {
    None => zk_ssl_air::juez::juzgar::<stark_experiment::juez::EnvioV1>(&receipt.proof, pi),
    Some(_) => zk_ssl_air::juez::juzgar::<stark_experiment::juez::EnvioV2>(&receipt.proof, pi),
}
.map_err(|r| LayerError::VerificationFailed(format!("envio: {r:?}")))?;
```

#### Boceto 2 — Verificar en paralelo y aplicar en serie, sin clonar (REND-03, REND-04, D1, D2)

Estado: **compila** en su sitio con el lifetime nombrado (`cargo check -p zk-ssl`, en release y
en dev, sin avisos); con `'_` en el cierre no compilaba («lifetime may not live long enough»). Con
eso el compilador confirma que `SovereignLayer`, `SparseTree`, `BatchOp` y `PlanListo` cumplen
`Sync` y `Send`. La prueba de equivalencia serie = paralelo no está escrita.

```rust
// crates/zk-ssl/src/two_phase.rs, apply_many_con_operacion: pasos 2 y 3 (hoy, líneas 1486-1534).
// La firma nombra el lifetime, sin cambiar la API pública:
//   pub fn apply_many_con_operacion<'a>(&mut self, ops: &[BatchOp<'a>]) -> Result<(), (usize, LayerError)>
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

// 2 · sin instantánea: nada muta hasta el paso 4, así que el estado vivo ES la instantánea (REND-03).
let (acc, pend, froz) = (&self.accounts, &self.pending, self.frozen.root());
#[cfg(debug_assertions)]
let raices_antes = (acc.root(), pend.root(), froz);

// 3 · validar en paralelo (REND-04). Puro (&self): el veredicto es el del MENOR índice que falla,
//     como en serie (RFC-0014: manda la primera que falla). Exige `SovereignLayer: Sync`.
let this: &SovereignLayer = &*self;
let primer_fallo = AtomicUsize::new(usize::MAX);
let validar = |j: usize, op: &BatchOp<'a>| -> Option<Result<PlanListo<'a>, LayerError>> {
    if j > primer_fallo.load(Ordering::Relaxed) { return None; } // ya falló una anterior (D1)
    let r = catch_unwind(AssertUnwindSafe(|| match op {
        BatchOp::Send { receipt, sender_index, sender_state, amount } => this
            .validate_send(acc, pend, froz, receipt, *sender_index, sender_state, *amount)
            .map(|p| PlanListo::Send(p, *receipt)),
        BatchOp::Claim { receipt, receiver_index, receiver_state, notice } => this
            .validate_claim(acc, pend, froz, receipt, *receiver_index, receiver_state, notice)
            .map(|p| PlanListo::Claim(p, *receipt)),
    }))
    // cinturón general: un hilo que cae no tumba el lote ni pierde su índice (fallar cerrado)
    .unwrap_or_else(|_| Err(LayerError::VerificationFailed("pánico al validar".into())));
    if r.is_err() { primer_fallo.fetch_min(j, Ordering::Relaxed); }
    Some(r)
};
let hilos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(ops.len());
let trozo = ops.len().div_ceil(hilos);
let mut resultados = Vec::with_capacity(ops.len());
std::thread::scope(|s| {
    let manos: Vec<_> = (ops.chunks(trozo).enumerate())
        .map(|(k, c)| {
            let validar = &validar;
            s.spawn(move || c.iter().enumerate().map(|(i, op)| validar(k * trozo + i, op)).collect::<Vec<_>>())
        })
        .collect();
    for m in manos { resultados.extend(m.join().expect("cada pánico ya se recogió dentro del hilo")); }
});
let mut planes = Vec::with_capacity(ops.len());
for (j, r) in resultados.into_iter().enumerate() {
    match r {
        Some(Ok(p)) => planes.push(p),
        Some(Err(e)) => return Err((j, e)),
        // solo se salta lo que va DETRÁS de un fallo ya evaluado, y el bucle lo encuentra antes
        None => unreachable!("índice saltado sin un fallo anterior"),
    }
}
#[cfg(debug_assertions)]
debug_assert_eq!(raices_antes, (self.accounts.root(), self.pending.root(), self.frozen.root()));
// 4 · aplicar en serie, como hoy, y persistir una vez por lote (REND-05, corte 4).
```

#### Boceto 3 — Las fronteras incrementales de época (REND-01, REND-02, D1, D2)

Estado: **compila** en su sitio (`cargo check -p zk-ssl -p zk-ssl-verify`). La revisión encontró
que `FronteraDensa` daba la raíz del árbol vacío con el árbol lleno (n = 2^d); con la corrección
que lleva ahora, un test en la copia la iguala a `SparseTree` para d ∈ {1, 2, 3, 5, 7} y todo n de
0 a 2^d, y con saltos (MEDIDO). `CimasMmr` no compilaba por un `use` duplicado; sin él, coincide
con `mmr::cima` para t de 1 a 300 (MEDIDO).

```rust
// crates/zk-ssl/src/sparse_tree.rs — la MISMA raíz que SparseTree::with_depth(d) con hojas en 0..n,
// también con el árbol lleno (n = 2^d), sin guardar hojas ni nodos (acumulador de rama izquierda).
// `native_merge` y `Digest` ya se importan en este fichero (línea 53).
pub struct FronteraDensa {
    profundidad: usize,
    vacio: Vec<Digest>, // idéntico a `empty` de with_depth (sparse_tree.rs:95-100)
    rama: Vec<Digest>,  // rama[k]: el subárbol izquierdo completo de altura k; rama[d], la raíz llena
    n: u64,
}
/// `NoCreciente` es el mismo caso que hoy da `RxRepetido` en vista_recibos (D1, D2).
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorFrontera { Llena, NoCreciente { indice: u64, siguiente: u64 } }

impl FronteraDensa {
    pub fn nueva(profundidad: usize) -> Self {
        let cero: Digest = [BaseElement::ZERO; 4];
        let mut vacio = vec![cero];
        for k in 1..=profundidad { let p = vacio[k - 1]; vacio.push(native_merge(p, p)); }
        Self { profundidad, vacio, rama: vec![cero; profundidad + 1], n: 0 }
    }

    /// Amortizado: un merge por hoja. La última hoja deja la raíz completa en rama[profundidad].
    pub fn anadir(&mut self, hoja: Digest) -> Result<(), ErrorFrontera> {
        if self.n >= 1u64 << self.profundidad { return Err(ErrorFrontera::Llena); }
        let (mut nodo, mut i) = (hoja, self.n);
        for k in 0..=self.profundidad {
            if i & 1 == 0 { self.rama[k] = nodo; break; }
            nodo = native_merge(self.rama[k], nodo);
            i >>= 1;
        }
        self.n += 1;
        Ok(())
    }

    /// Recibos: cada rx saltado (un número quemado) entra como hoja vacía, que es el digest cero,
    /// así que la raíz coincide con la de SparseTree.
    pub fn anadir_en(&mut self, indice: u64, hoja: Digest) -> Result<(), ErrorFrontera> {
        if indice < self.n { return Err(ErrorFrontera::NoCreciente { indice, siguiente: self.n }); }
        while self.n < indice { self.anadir([BaseElement::ZERO; 4])?; }
        self.anadir(hoja)
    }

    /// `profundidad` merges: la raíz de la época en curso.
    pub fn raiz(&self) -> Digest {
        if self.n == 1u64 << self.profundidad { return self.rama[self.profundidad]; }
        let (mut nodo, mut i) = (self.vacio[0], self.n);
        for k in 0..self.profundidad {
            nodo = if i & 1 == 1 { native_merge(self.rama[k], nodo) } else { native_merge(nodo, self.vacio[k]) };
            i >>= 1;
        }
        nodo
    }
}
// En el nodo: DOS fronteras de acuses (la de la época que se firma y la abierta), indexadas por su
// seq de inicio. La vieja se retira en `conservar` (latido.rs:425-451) cuando avanza ultima_cabeza,
// nunca al componer; si `latir` falla, sigue viva. La de recibos se alimenta solo cuando `anotar()`
// devuelve Ok, y antes de firmar se contrasta desde disco, fuera del candado.
// Compuerta (D2): frontera == raiz_de_epoca == nucleo.py, con huecos de rx y con el árbol lleno a
// una profundidad pequeña.

// crates/zk-ssl-verify/src/mmr.rs — junto a `cima`: un productor, contrastado con el de referencia.
// (mmr_hoja, mmr_nodo y Digest ya se importan en mmr.rs:42.)
#[derive(Default)]
pub struct CimasMmr { cimas: Vec<(u32, Digest)>, t: u64 } // (altura, raíz perfecta), izq. a der.

impl CimasMmr {
    pub fn anadir(&mut self, cabeza: Digest) {
        let (mut nodo, mut h) = (mmr_hoja(cabeza), 0u32); // con mmr_hoja, como mth (mmr.rs:63-80)
        while let Some(&(hh, izq)) = self.cimas.last() {
            if hh != h { break; }
            self.cimas.pop();
            nodo = mmr_nodo(izq, nodo);
            h += 1;
        }
        self.cimas.push((h, nodo));
        self.t += 1;
    }
    /// El MTH de RFC 6962 es el plegado de derecha a izquierda de las cimas perfectas.
    pub fn cima(&self) -> Option<Digest> {
        let mut it = self.cimas.iter().rev();
        let mut acc = it.next()?.1;
        for &(_, izq) in it { acc = mmr_nodo(izq, acc); }
        Some(acc)
    }
}
// Compuerta (D2): para t en 1..=2^12, CimasMmr::cima == mmr::cima(&hojas[..t]) == nucleo.cima(.., con_hoja=True).
```

#### Boceto 4 — El instrumento que ata el nivel declarado (ZK-1, ZK-2, D4)

Estado: **compila y pasa** en su sitio
(`cargo test --release -p zk-ssl --lib las_opciones_de_la_casa`). Imprime `SEGURIDAD conjeturada=127 ldr=80 udr=59` (MEDIDO).

```rust
// crates/zk-ssl/src/metrics.rs, mod tests: una prueba OCULTA real, del mismo montaje que
// el_coste_de_verificar_una_prueba (metrics.rs:247). Sin cable, sin kit, sin segunda implementación.
// No es #[ignore]: es una comprobación de ≈1,3-1,6 s (MEDIDO: 1,27-1,57 s, cuatro ejecuciones
// en release y 4 núcleos) con su pin en el canon.
#[test]
fn las_opciones_de_la_casa_dan_lo_declarado() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    let bob = open_and_fund(&mut layer, SK_BOB, 0);
    let estado_a = state_of(&layer, alice);
    let receptor = layer.public_id_of(bob).expect("cuenta");
    let envio = layer
        .send(BaseElement::new(SK_ALICE), alice, &estado_a, receptor, salt_de(0x5EC), 250_000)
        .expect("envio");
    let p = winterfell::Proof::from_bytes(&envio.proof).expect("prueba");
    // La demostrable depende de la longitud de la traza: tiene que ser la OCULTA (2T).
    assert_eq!(p.trace_info().length(), 2 * stark_experiment::circuit_send::TRACE_LENGTH);
    let conj = p.conjectured_security::<Blake3>().bits();
    let dem = p.proven_security::<Blake3>();
    // Una línea que check_publicadas pueda atar: PRINCIPIOS, lib.rs:212-215, INSTITUCIONAL y
    // FIVE_BACKENDS se atan a ESTA salida, no a una réplica (regla de los dos productores).
    eprintln!("SEGURIDAD conjeturada={conj} ldr={} udr={}", dem.ldr_bits(), dem.udr_bits());
    // Umbrales: los que hoy da la réplica. Si la función real da menos, es un hallazgo, no un ajuste.
    assert!(conj >= 127, "conjeturada {conj}");
    assert!(dem.ldr_bits() >= 80, "LDR {} (UDR {})", dem.ldr_bits(), dem.udr_bits());
}
```

#### Boceto 5 — El error tipado del SDK (ARQ-03, D3)

Estado: **compila** en su sitio (`cargo check --release -p zk-ssl-sdk`, sin avisos). La revisión
lo compiló con un aviso de `dead_code` en `es_estado_viejo`; lleva ahora el `allow` y las `impl`
de `Display` y `Error` que necesita el envoltorio `call`.

```rust
// crates/zk-ssl-sdk/src/lib.rs — `call` (lib.rs:59-82) pasa a ser un envoltorio de `call_tipado`.
#[derive(Debug)]
pub enum ErrorNodo {
    Transporte(String),
    /// -32000: la capa rechazó. `causa` = data.causa (§454). `recepcion_afirmada` = data.recepcion
    /// TAL COMO LLEGÓ, cruda como la `Constancia` del §606: la afirmación de un nodo que puede
    /// mentir (RFC-0011). No es evidencia hasta que el kit la juzga contra la cabeza firmada; el
    /// SDK no la verifica ni la reescribe (D3 y su réplica).
    Rechazo { mensaje: String, causa: Option<String>, recepcion_afirmada: Option<Value> },
    Credencial(String),                     // -32004
    Parametros(String),                     // -32602
    Interno { code: i64, mensaje: String }, // -32603 (PARADA incluida) y cualquier otro
    Respuesta(String),
}

// Para que `call` (anyhow::Result) envuelva a `call_tipado` con `?`.
impl std::fmt::Display for ErrorNodo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ErrorNodo {}

impl ErrorNodo {
    /// spec/RPC.md:164-165 dice «refresca su vista y reintenta», y reintentar exige regenerar la
    /// prueba: si el primer intento se aplicó y se perdió la respuesta, se paga dos veces. Por eso
    /// no se hace pública hasta que exista la consulta de ARQ-07 (zkssl_estadoDePrueba).
    #[allow(dead_code)] // sin llamador hasta ARQ-07
    pub(crate) fn es_estado_viejo(&self) -> bool {
        matches!(self, ErrorNodo::Rechazo { causa: Some(c), .. } if c == "StaleState")
    }
}

impl Rpc {
    pub fn call_tipado<P: Serialize, R: DeserializeOwned>(&self, method: &str, params: P) -> Result<R, ErrorNodo> {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let resp: Value = (self.agent.post(&self.url).send_json(body))
            .map_err(|e| ErrorNodo::Transporte(format!("{method}: {e}")))?
            .into_json()
            .map_err(|e| ErrorNodo::Respuesta(e.to_string()))?;
        if let Some(err) = resp.get("error") {
            let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
            let mensaje = err.get("message").and_then(Value::as_str).unwrap_or("").to_string();
            let data = err.get("data");
            return Err(match code {
                -32000 => ErrorNodo::Rechazo {
                    causa: data.and_then(|d| d.get("causa")).and_then(Value::as_str).map(str::to_string),
                    recepcion_afirmada: data.and_then(|d| d.get("recepcion")).cloned(),
                    mensaje,
                },
                -32004 => ErrorNodo::Credencial(mensaje),
                -32602 => ErrorNodo::Parametros(mensaje),
                _ => ErrorNodo::Interno { code, mensaje },
            });
        }
        let r = resp.get("result").cloned().ok_or_else(|| ErrorNodo::Respuesta(format!("{method}: sin result")))?;
        serde_json::from_value(r).map_err(|e| ErrorNodo::Respuesta(e.to_string()))
    }
}
```

---

## 6. Descartes

| idea | razón | fuente |
|---|---|---|
| «Millones de pruebas de conservación por segundo» por nodo | Inalcanzable por cadena de raíces (≈1.300 op/s de cota) e innecesario (un RTGS pide 21-105 op/s). 10^6/s piden ≥ ≈770 cadenas, ≈4.000-4.900 núcleos solo para verificar y ≈0,82-0,87 millones de núcleos de cliente para generar | encargo genérico; §1 de este documento |
| Compromisos de Pedersen para saldos y notas | Logaritmo discreto: Shor los rompe, y chocan con la tesis poscuántica. La suma ya la cubre el AIR de cada operación (partida doble) | encargo genérico; `crates/stark-experiment/src/circuit_send.rs:1091` |
| Vector commitments algebraicos (Verkle sobre KZG o IPA) | KZG exige emparejamientos y SRS (ceremonia); IPA, logaritmo discreto. Los de retículos no tienen implementación auditada en Rust y sus aperturas superan un camino de 32 hashes | encargo genérico |
| SNARK Groth16 o PLONK-KZG, envolver el STARK en Groth16, o Nova como motor detrás de un trait | Ya medidos (`FIVE_BACKENDS.md`: 192 B y 1.008 B, con setup). Ceremonia y curvas en la vía de producción; el tamaño no bloquea la mensajería. Se conservan como evidencia, no como alternativa | encargo genérico; `FIVE_BACKENDS.md`; especialista de arquitectura |
| Bulletproofs para los rangos | Logaritmo discreto y verificación lineal. El rango de 63 bits ya se demuestra en el AIR, en paralelo a las subidas de hash, sin alargar la traza | encargo genérico |
| «Zero-sum proofs» sobre compromisos homomórficos | Necesitan Pedersen. En un libro de un solo escritor la suma cero ya es una restricción del AIR de cada operación; agregarla por época no añade solidez y añade un circuito | encargo genérico |
| Recursión o agregación (RISC Zero, Plonky3, Miden) | RISC Zero medido: recibo de 223.234 B constante con 40,9 s de prueba y 23,8 s de compresión en GPU (§306). Miden descartada por zkVM; Plonky3 sería migrar los AIR: 28 circuitos al escribirse [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.4.bis, y hoy 35 (censo de RFC-0009 D-G). Solo encaja el cobro agregado de un mismo titular (ZK-7) | encargo genérico |
| Delegar la generación a GPUs, optimizar MSM, agregar con Nova o Halo2 | Prueba el cliente y el testigo lleva material de la clave de gasto (DIAGNOSTICO §2.2); no hay MSM en producción (DIAGNOSTICO §2.1); curvas | encargo genérico |
| Circle STARK, Stwo, campos de 31 bits (M31, BabyBear) | QM31 da ≈124 bits (ESTIMADO: 4 × 31), por debajo de los 127 declarados; cambiar el campo cambia Rescue-Prime sobre Goldilocks, que es núcleo (`spec/NUCLEO.md:248`, §6); y con 31 bits la colisión de identidades baja a 2^15 y el rango de 63 bits quedaría en 30 (`doc/integracion-vertical-evaluacion.md:79`) | encargo genérico |
| Otro hash en el circuito (RPO, Poseidon2, Griffin, Anemoi); Rescue en FRI y Fiat–Shamir; Blake3-192 en el vector commitment | Rescue es núcleo congelado (`spec/NUCLEO.md` §2 y §6): cambiarlo es versión nueva del preámbulo y unos 380 vectores, sin ganancia en el AIR. Rescue en FRI solo sirve para la recursión, descartada, y multiplicaría el coste de Merkle en nativo. Blake3-192 da 96 bits de resistencia a colisiones, por debajo del nivel declarado | encargo genérico (otro hash, Rescue en FRI); especialista de criptografía |
| Plegado FRI 16 o 4; un solo árbol de Merkle para traza y composición, o caps; LogUp o batching algebraico | El modelo da más bytes con 16 o 4 que con 8 en este dominio (≈84,0 y ≈85,5 KB frente a ≈78,9, ESTIMADO). La composición depende de desafíos que se sacan después de comprometer la traza, y `BatchMerkleProof` ya poda los niveles altos. Los rangos no alargan la traza, y el batching algebraico resta bits demostrables | encargo genérico (plegado FRI); especialista de criptografía |
| Dos rondas de Rescue por fila (T = 512), o quitar la subida de congelados | La primera suma 24 columnas, sube la prueba ≈10 KB (ESTIMADO) y obliga a rehacer los 35 AIR y su FV: peor que ZK-3. La segunda deja 560 filas, que siguen en 1024, y el arreglo B quiere atarla, no retirarla | especialista de criptografía |
| Quitar o trocear el `Mutex` global | Lo que serializa es la raíz (§230): quitar el candado no cambia ningún resultado y añade superficie. El paralelismo útil está dentro del lote (REND-04) | encargo genérico; especialista de rendimiento |
| Verificar fuera del candado en la vía suelta | El contrato del RFC-0010 y la coherencia del latido (§570) dependen de lo que ocurre bajo el candado; bajo carga, la vía suelta muere por contención de raíz. Solo en lote | especialista de rendimiento |
| Cambiar `Rp64_256` por RPO o un hash no algebraico en los árboles de estado | Los caminos de cuentas y pendientes los sube el circuito: el hash nativo tiene que ser el del AIR | especialista de rendimiento |
| Paralelizar la subida de un único camino de Merkle | Dependencia de datos en serie; solo se paraleliza entre operaciones de un lote | especialista de rendimiento |
| Estructuras persistentes (HAMT, crate `im`) | Una dependencia más (principio 4); basta quitar el clon (REND-03) y los pendientes densos (REND-08) | especialista de rendimiento |
| Un circuito de lote | Descartado en el §210: ≈73 op/s (ESTIMADO allí: recta ajustada sobre dos geometrías medidas, cota optimista) frente a los ~320 del apply de entonces (banda 265-320, §217, en `spec/rfc/0002-lotes-y-transicion-de-hoja.md:313`) | §210 |
| Optimizar `settlement-layer` o cachear el árbol del medio | Fuera de la vía de producción; el medio difiere su decisión a E3 «si el publicador lo necesita, se mide» | especialista de rendimiento |
| Hasher multiplicativo sin clave en todos los árboles (REND-11 original) | Donde la posición no la asigna la capa, un hasher sin clave degrada las búsquedas bajo el candado. Concedido en la réplica | D1, D2 |
| Sustituir la derivación de los contadores custodiados por `meta:cust_uses` (REND-07, parte 2) | Reabre la clase del §393 y §394 | D1 |
| Una raíz de consumos por cabeza para servir `cons_path` en O(log) | Una raíz no da caminos | D2 |
| Reservar rx en bloques (REND-05, opción 3) | Retirado en la réplica: haría indistinguible un número quemado de una anotación perdida y cambia la forma del árbol de recibos que comprueba el kit | D1, D2, réplica de rendimiento |
| La firma XMSS del diario en binario | Aplazado: el testigo lee el diario; cambiar su formato va por RFC | D1, réplica de REND-02 |
| Una medida estadística (χ²) de los valores abiertos | ≈23-25 min por corrida y poco poder de falsación; retirada en la réplica, también como banco fuera del canon | D4, réplica de criptografía |
| Un trait de motor «universal» que abstraiga también campo y hash | Goldilocks, `Rp64_256`, `digest_to_bytes` y `epoch_digest` son núcleo congelado (RFC-0005 D-A) | especialista de arquitectura |
| Reutilizar `SettlementProver` como trait de producción | Modela las ceremonias del estudio comparativo; no tiene forma de traza, opciones ni familia; cuesta más que `Juez` | especialista de arquitectura |
| Fusionar los AIR v1 y v2 | Los dos están vivos; el 0.2 tiene que seguir idéntico byte a byte y el guardián de layout barre por fichero | especialista de arquitectura; `crates/stark-experiment/src/circuit_send_v2.rs:1-35` |
| Separar testigo, cable o comparativos con features | Las puertas de clausura leen `Cargo.lock`, que incluye las dependencias opcionales, y `--all-features` lo compila todo (RFC-0011 D-A) | especialista de arquitectura |
| Dar biblioteca al nodo | Decidido en contra en el §598 (reversible); sin llamador medido | especialista de arquitectura, corregido por su verificador |
| Bindings C, Java o Python del probador | Arrastraría `winter-prover` y la custodia de claves por FFI; lo que un tercero corre es el verificador (ARQ-06) | especialista de arquitectura |
| Otros transportes (gRPC, colas, websockets push) | Ningún cuello de transporte medido; duplica DTO y vectores sin llamador | especialista de arquitectura |
| Borrar los crates comparativos, o renombrar `stark-experiment` | BACKLOG 56: los comparativos se conservan como evidencia. Renombrar toca siete `Cargo.toml` y cientos de citas sin ganancia (precedente del §255); basta corregir la cabecera | especialista de arquitectura |
| Un DSL o macro de circuitos | Evaluado y declarado FALSO en `doc/integracion-vertical-evaluacion.md` §2 fila 14 | especialista de arquitectura |
| Un `Almacen` en memoria publicado como segundo llamador | Llamador artificial: el criterio del §580 es no conectar nada hasta que haya un operador real; solo para tests | verificador de ARQ-05 |
| Un segundo workspace en el mismo repositorio para los comparativos | `doc/MTC.md:12-15` llama «parche» a esa forma y eligió extraer; queda como P3 opcional y declarada | verificador de ARQ-08 |
| Un cursor nuevo para `zkssl_logEntries` | Ya existe: `{fromSeq, limit}` (`spec/RPC.md:88`) | verificador de ARQ-07 |
| Buscar carreras entre verificar y aplicar en el nodo | No las hay en la vía actual: `dispatch` mantiene el candado durante toda la petición y el lote valida contra una instantánea antes de mutar | especialista de seguridad |
| Tratar como fuga la variación del tamaño de prueba frente al importe | Superada por medida: con los importes barajados en 12 corridas, 192 muestras, p 0,64 y 0,65 (BACKLOG 114, §620-§621); en envío y cobro el importe ya es entrada pública | especialista de seguridad |
| Buscar bombas de memoria al leer pruebas | Cerrado en el §575 (la lectura de una prueba no reserva lo que sus bytes no traen); el §578 midió que en las vías vivas no queda residuo | encargo genérico; especialista de seguridad |

---

## 7. Lo que este documento NO hace, lo que NO cierra, y dónde tiene menos confianza

### 7.1 Lo que NO hace

- **No describe los ocho hallazgos en embargo**, ni su componente, ni su mecanismo, ni su arreglo.
  Su detalle está en manos del autor. Allí donde una propuesta pública dependía de uno de ellos, se
  publicó solo la parte que se sostiene sin él; eso puede dejar algún corte con menos condiciones
  de las que el debate le puso, y el autor las tiene.
- **No deja los bocetos en el árbol.** Se compilaron insertados en su sitio en una copia
  desechable de `e69fadd`, fuera del repositorio (§5.4), y allí corrieron el boceto 4 y los tests
  de equivalencia del boceto 3. Nada de eso es todavía un test del canon.
- **No mide nada de lo propuesto.** Toda la columna «Propuesta» del §5.1 es ESTIMADO o SUPUESTO.
- **No cambia código ni el BACKLOG.** Es un fichero nuevo en `doc/`, y entra con el asiento que lo
  registra (§637 de [`AUDITORIA.md`](../AUDITORIA.md)). Abrir entradas, elegir cortes y decidir son
  del autor.
- **No sustituye una auditoría externa.** Diecisiete agentes de una misma sesión no son diecisiete
  revisores independientes: comparten sesgos, y los verificadores leyeron lo mismo que los
  especialistas.

### 7.2 Lo que NO cierra

- ⚠️ **Los ocho hallazgos en embargo**, hasta que el corte 0 esté en `main`.
- ⚠️ **La suficiencia del AIR**: FV-2 se ejecutó solo sobre `circuit_refund` y FV-3 sigue abierta
  (`doc/VERIFICACION_FORMAL.md:286-290`). El censo de la frontera de SEC-3 cubre otra clase.
- ⚠️ **El argumento de simulación de la ocultación** (ZK-6): es trabajo de especificación sin
  auditoría externa del fork.
- ⚠️ **La custodia de la clave XMSS**: el cerrojo de SEC-1 cubre dos procesos sobre el mismo
  contador en la misma máquina. La misma semilla con otro contador, o en otra máquina, sigue siendo
  solo detectable, por los testigos (§331, §599).
- ⚠️ **Lo que no es criptografía** para un RTGS: alta disponibilidad, recuperación ante desastres,
  continuidad operativa auditada, certificación y firmeza jurídica
  ([`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md) §6.1; su §6.6 deja la firmeza, la
  certificación, la gobernanza y la continuidad auditada fuera de lo que un sello resuelve).
  Ningún corte lo toca.
- ⚠️ **«Millones por segundo»**: el plan no lo persigue, y ningún corte lo alcanza.

### 7.3 Dónde tiene menos confianza

- **Las medidas de la sesión** son de una máquina en la nube de 4 vCPU con otros agentes corriendo
  (load average 1,6-2,8): la mediana de generar (865 ms) queda por encima de los mínimos del §538
  (698-742 ms, otra máquina), y la de verificar se movió entre 3,98 y 4,93 ms de un proceso a otro.
  Valen como orden de magnitud y como contraste del debate, no como pin.
- **La seguridad demostrable** (LDR 80, UDR 59) salió de una réplica en Python de `security.rs`;
  la función real, corrida una vez con el boceto 4 en una copia desechable de `e69fadd`, da lo
  mismo, pero ningún test del árbol la ata todavía. **Los bytes** de ZK-1, ZK-3 y ZK-5 salen de un
  modelo del especialista que no está en el árbol; para la sal, el spike de D-I midió 6.567 B, más
  que los ≈5,3 KB del modelo.
- **La ganancia de ocultar en la holgura** (ZK-3) depende de un reparto del probador que nadie ha
  medido: atribuir el exceso a la duplicación 2T es un SUPUESTO hasta tener los spans.
- **Las cuentas de Amdahl del nodo** (≈103 merges en serie, ≈1.300 op/s) usan 7,44 µs por merge
  «implícito», que incluye el `HashMap`, y caen dentro de la banda del §217 B.3; **el clon a escala**
  (REND-03) extrapola una corrida sin aparear de árboles de 16k nodos.
- **El margen a pico RTGS con disco** (§1) es la conclusión más útil del documento y también una
  estimación sin medir: suma al apply de la capa, medido en la sesión, el RPC, los fsync y el flush
  medidos en otra máquina (§229, §234, §204). El corte 1 la convierte en medida o la tira.
- **Los identificadores** son propios de este documento: quien lo cruce con el expediente del
  enjambre necesita la tabla del autor.

---

## Fuentes

- El expediente público del enjambre (especialistas, verificadores, debates D1-D5 y réplicas), sin
  lo embargado; no está versionado. Las medidas de la sesión, transcritas en el §2.1, y las de la
  revisión de los bocetos (§5.4). El árbol en `e69fadd`, con las citas `fichero:línea` comprobadas
  con `sed` y `grep` al redactar y otra vez al corregir.
- [`AUDITORIA.md`](../AUDITORIA.md): §16.7, §120, §204, §210, §217, §219, §221, §222, §229, §230,
  §234, §238, §255, §292, §306, §312, §331, §390, §393, §394, §417-§422, §427-§431, §442, §454,
  §463, §487, §512, §529, §538, §543, §570, §575, §578, §580, §589, §598, §599, §606, §620, §621,
  §629, §631 y §632.
- [`DIAGNOSTICO_ESCALADO.md`](DIAGNOSTICO_ESCALADO.md), [`ESCALADO.md`](ESCALADO.md),
  [`VERIFICACION_FORMAL.md`](VERIFICACION_FORMAL.md),
  [`integracion-vertical-evaluacion.md`](integracion-vertical-evaluacion.md), [`MTC.md`](MTC.md),
  [`INSTITUCIONAL.md`](INSTITUCIONAL.md); [`ARQUITECTURA.md`](../ARQUITECTURA.md),
  [`PRINCIPIOS.md`](../PRINCIPIOS.md), [`SECURITY.md`](../SECURITY.md),
  [`FIVE_BACKENDS.md`](../FIVE_BACKENDS.md), [`BACKLOG.md`](../BACKLOG.md),
  [`GENAI.md`](../GENAI.md), [`CONTRIBUTING.md`](../CONTRIBUTING.md);
  [`spec/RPC.md`](../spec/RPC.md), [`spec/NUCLEO.md`](../spec/NUCLEO.md),
  [`spec/rfc/0002-lotes-y-transicion-de-hoja.md`](../spec/rfc/0002-lotes-y-transicion-de-hoja.md),
  [`spec/rfc/0009-lo-que-revela-una-prueba.md`](../spec/rfc/0009-lo-que-revela-una-prueba.md),
  [`spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md`](../spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md).
- Repositorio: https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers
