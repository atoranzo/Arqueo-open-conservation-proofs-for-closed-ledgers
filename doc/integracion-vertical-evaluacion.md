# Evaluación: la «integración vertical» y un «módulo criptográfico Arqueo» que sea el estándar de su nicho

**La pregunta, tal como se hizo** (01-10-2026): «estudiar la integración vertical y desarrollar un
módulo criptográfico Arqueo cogiendo lo que sea útil de winterfell y del ecosistema para que sea el
estándar de su nicho/segmento», acompañada de un borrador largo —cuatro capas, una matriz de
selección, una arquitectura de cinco crates, código Rust, un DSL, un formato de prueba, una tabla de
simulación, parámetros de seguridad y un plan de estandarización con consorcio— que el autor aportó
tal cual.

**Respuesta corta.** La integración vertical que el nicho premia **ya está hecha** en las capas que
Arqueo controla: el dominio, la aritmetización y, sobre todo, **el lado del verificador**, que es lo
que un estándar de evidencia necesita y lo que `spec/NUCLEO.md` congela. En la capa del protocolo de
prueba existe en la medida exacta del fork de winterfell: tres ficheros nuevos y 350 líneas. Bajar a
la capa de las primitivas —campo, hash, FFT, GPU— **contradice tres decisiones medidas de la casa**
y no resuelve ningún cuello de botella que el árbol tenga medido. El borrador no sirve como base:
su código no compila contra el winterfell del árbol, el campo que elige agrava un hallazgo medido,
sus cifras no tienen medición detrás y su «aplicación estrella» es, palabra por palabra, lo que el
README dice que Arqueo no prueba. Lo que **sí** sale de las piezas de la casa es más pequeño y
medible, y se dice por su nombre en §5: nombrar el módulo que ya existe, la segunda implementación
que la entrada 85 pide desde hace meses —**hecha en su primer hito**: 26 de 26 KAT del núcleo en
Python, desde la spec—, el verificador en WebAssembly —que **verifica hoy** los 316 vectores de los
diez manifiestos bajo wasmtime, con el mismo veredicto que el nativo— y la decisión de capa 2 por
RFC con las cifras nuevas.

**Estado**: **ACEPTADA CON CORRECCIONES** por el autor el 07-10-2026, en el §705 de
[`AUDITORIA.md`](../AUDITORIA.md). Las correcciones van en el bloque de abajo, «Correcciones
(§705)», y mandan donde discrepen del texto. Al escribirse: evaluada, y los dos spikes de §5.2 y
§5.3 **ejecutados en la misma sesión** con su resultado dentro (`tools/segunda/`). Sin código nuevo
en `crates/`. Medido sobre `e8ac246` (S615) en la máquina de la sesión, rustc 1.97.0.

**Procedencia**: escrita en una sesión de trabajo con asistencia de IA generativa, en una rama que no
se ha fusionado (`claude/nice-planck-ax35zl`), sobre un borrador que también salió de un modelo y
que el autor pegó para contrastarlo, como los de `doc/ecst/`. Integrar no es aceptar: la aceptación
que describe [`GENAI.md`](../GENAI.md) quedó pendiente hasta el §705, que la da con correcciones. No
toca `BACKLOG.md`, `AUDITORIA.md` ni `NOTICE`: eso es del autor.

**Convención**: ✅ medido en este árbol, con el comando al lado · 📐 derivado con una aritmética que
se reproduce · 📖 leído en la fuente primaria, con cita; cuando la fuente es GitHub o crates.io, con
la fecha de lectura (01-10-2026) · ⚠️ matiz · **CONFIRMADA / PARCIAL / FALSA / SIN FUENTE /
CONTRADICE LO MEDIDO**, los veredictos de `doc/ecst/VERIFICACION.md`, más uno propio, **YA HECHO**,
para lo que el borrador propone como futuro y el árbol lleva sellado.

---

## Correcciones (§705)

**Aceptada con correcciones.** El autor la acepta el 07-10-2026, en el §705 de
[`AUDITORIA.md`](../AUDITORIA.md), con alcance «en este repositorio y en el proyecto NLnet»: el
árbol de este repositorio y los hitos de la propuesta 2026-11-009 a NLnet
(`spec/rfc/0005-nucleo-congelado.md:26`). El cuerpo no se reescribe: este bloque manda donde
discrepen. Separa los errores, que ya lo eran en `e8ac246`, de las cifras de foto, ciertas entonces
y envejecidas después. Lo de hoy se midió sobre `c990868`, el commit anterior al §705. Las
secciones y filas son las de este documento.

**Lo que se acepta**, con el fundamento corregido de abajo:

- las capas 4 y 3 y el lado del verificador ya son de la casa, y el estándar está en lo que un
  tercero reproduce (§4);
- la capa 2 no se decide aquí: antes, un spike y un RFC (§4 y §5.4), todavía pendientes;
- capa 1, no (§4 y §6);
- el borrador no es base de nada (§3 y §6);
- el módulo se nombra con un mapa, sin crate (§5.1), y de las piezas solo sale `kat_xmss` (§5.5).

El alcance cubre, por su nombre, tres veredictos de §6: «Capa 1: no», «Capa 2: decidir por RFC» y
«un módulo criptográfico nuevo: no; nombrar el que existe».

### Errores, que ya lo eran en `e8ac246`

| # | dónde | dice | corrección | evidencia |
|---|---|---|---|---|
| E1 | §1, fila «1 · Primitivas»; §2, fila 1 | 68 ficheros usan `fields::f64` | **89**, y ninguna de las variantes probadas del grep —con el fork, en todo el árbol, por líneas o solo en `src/`— da 68; hoy, 90. `fields::f128`, 3 | `git grep -l 'fields::f64' <rev> -- crates/ ':!crates/winter-*' \| wc -l`: 89 en `e8ac246`, 90 en `c990868` |
| E2 | respuesta corta; §1, fila «2 · Protocolo»; §6 | el fork son «350 líneas», «tocadas en el probador y el verificador» | el 350 es la **estimación** del spike de RFC-0009 D-F («488 líneas —350 en el fork y 138 propias», `spec/rfc/0009-lo-que-revela-una-prueba.md:212-213`), no una medida. Medido: +813/−6 en `e8ac246`; hoy **+829/−6** en 14 ficheros, 11 tocados y 3 nuevos, y **617** sin blancos ni comentarios. El probador y el verificador solos, +500 brutas y 399 netas. Sigue por debajo del umbral de 1.784 con el que D-F se revierte (`:217-219`) | `git diff --no-index --numstat` de `src/` contra los `.crate` 0.13.1 de crates.io, con el sha256 de la línea 3 de `crates/winter-*/Cargo.toml`: `winter-air`, 7 ficheros, +329/−3; `winter-prover`, 6, +427/−2; `winter-verifier`, 1, +73/−1. `NOTICE:39-40` |
| E3 | §2, fila 9; Fuentes | «BACKLOG 92» para la GPU suspendida | es la **entrada 22**; la 92 es la custodia de la clave | `BACKLOG.md:2019` (22, SUSPENDIDA), `:2065` (RTX 5090), `:2092` (12,4 GiB); `:956` (92), hasta el §705; desde el §706, `:2024`, `:2070`, `:2097` y `:961`; desde el §707, `:2043`, `:2089`, `:2116` y `:961`; desde el §708, `:2048`, `:2094`, `:2121` y `:961` |
| E4 | respuesta corta; §2, fila 2; §4, «Capa 1»; §7 | el campo de 31 bits «agrava un hallazgo medido»: la colisión de identidades en 2³² | ese hallazgo **ya estaba tachado** como corregido: la identidad es un digest de 256 bits. Lo vivo es el espacio de claves, 2⁶⁴ (§82) | `BACKLOG.md:1401-1408` hasta el §705; desde el §706, `:1406-1413`; desde el §707, `:1425-1432`; desde el §708, `:1430-1437`, en la entrada 15 |
| E5 | §2, fila 4; §4, «Capa 1» | la única decisión de capa 1 con número es `digest_of_proof` con Rescue (30,99 ms, techo del nodo) | está **hecha desde el §209** (`zkssl/0.2`): `digest_of_proof` es Blake3. Los 30,99 ms son del §204, anterior | `crates/zk-ssl-hash/src/lib.rs:1716`, `:1724` y `:1737-1739` |
| E6 | §1, tras la tabla; §4, «Capa 1» | «una regla escrita en la capa 1»: «escribir criptografía propia aquí sería un error grave» | es un **comentario local** que justifica una dependencia, `chacha20poly1305`, la del cifrado en reposo; no es una regla del proyecto. El criterio general escrito es el de `CONTRIBUTING.md:149-157`: no apilar superficie sin un problema presente y medido | `crates/zk-ssl/Cargo.toml:35-37` |
| E7 | §2, fila 11 | la casa declara 127 conjeturados, y 128 demostrables costarían «125,6 KB en vez de 36,7» | esas cifras, como los «29-63» demostrables, son del circuito de comparación, con 32 consultas y sin molienda, no de producción; la casa las atribuía a producción hasta el §698. Producción —42 consultas, blowup 16, molienda 21, extensión cuadrática— da sobre las pruebas ocultas 127 conjeturados, UDR 59 y LDR 80-88 según el circuito, menos en la edad con `m` alta; desde el §708, la conjeturada con el término DEEP, que su fórmula no descuenta: 112,8-116,8 según el circuito | `crates/stark-experiment/src/compliance_real_proof.rs:211-220`; los tests del §697; `SECURITY.md` §3.11 |
| E8 | §2, fila 17 | «el tamaño [del WASM] no se ha medido» | se contradice con §5.3, que lo mide: 1.587.916 B | §5.3, el bloque de órdenes y la tabla |
| E9 | §2, filas 16 y 19; §4, «Capa 1»; §5.4 | `VISION.md` (P4, P6, P8 y §5) como criterio vigente | `VISION.md` se declara **histórico**: la posición vigente está en `README.md` y `doc/USE_CASES.md`. Se cita como antecedente, no como regla | `doc/historia/VISION.md:1-4` |
| E10 | §5.1; §6 | «§580: la casa no cría código sin llamador» | el §580 hace lo contrario: declara `podar` **sin llamador** y la conserva por decisión del autor. La frase es de `spec/rfc/0012-el-ancla-de-cabezas.md:25` y `:121`, y vale como criterio de ese RFC, no como regla del §580 | `AUDITORIA.md` §580 |

Dos matices, sin rango de error:

- **«35 AIR»** (§1, §2 fila 14, §4 y §5.4) es el censo heredado de RFC-0009 D-F (`:211`, `:216`).
  El árbol no tiene un censo único: fuera del fork hay **38** `impl Air for`, 32 en
  `stark-experiment`, con falsadores y experimentos dentro, 5 en `zk-ssl-air`, que son las del kit,
  y 1 en `crates/zk-ssl/src/instrumento_edad.rs`. Se cita como «el censo de RFC-0009».
- **«Verificar cuesta 1,5 a 4 ms»** (§2, fila 8) es anterior a la ocultación del §538. El plano v2.0
  midió 3,98-4,93 ms (`doc/blueprint-v2.md` §1); aquí no se ha vuelto a medir.

### Cifras de foto: ciertas en `e8ac246`, envejecidas después

| dónde | en `e8ac246` | hoy | evidencia |
|---|---|---|---|
| respuesta corta; §5.2; §6 | 26 de 26 KAT del núcleo | **29 de 29** | `git ls-tree` de `spec/vectors/nucleo`; `python3 -B tools/segunda/juez_nucleo.py` |
| §1 | el kit `arqueo-verify-v0.2.0`; la clausura de `zk-ssl-verify`, 48 paquetes, y la de `zk-ssl`, 65 | **v0.4.2**; **61** y **68**: los tres de más en la capa son `argon2`, `base64ct` y `blake2`, que solo entran por el `argon2` del §702. Los otros tres crates, 17, 25 y 52, igual | `doc/KIT.md:19`; el método de §1, `cargo tree -p <crate> -e normal --prefix none --locked`; `cargo tree -i` |
| respuesta corta; §5.3; §7 | 316 vectores de diez manifiestos | **384 vectores en 12 manifiestos**: entran `ancla-cofirmada` (27) y `rotacion` (34), y crecen `paquete` (73), `conflicto` (17), `ancla` (22) y `completitud` (75). Son 399 líneas: las 15 que suma el §693 nombran la causa de vectores que ya estaban. Eso, hasta el §705; desde el §706, 390 vectores y 405 líneas: la edad y el pago ganan tres negativos cada uno. Que el WASM los pase hoy no se ha medido | los `MANIFIESTO.txt` de `FAMILIAS` (`tools/artefacto.sh:31`) |
| §5.2, tercer hito | `paquete` 70, `consumo` 14, `conflicto` 16 y `ancla` 21 | el segundo verificador pasa hoy `paquete` en sus 83 líneas, `consumo` 14, `conflicto` 17 y `ancla` 22, y además `rotacion` 34 | `tools/conformidad.sh tools/segunda/verificador.py`, como el bloque «3 duodecies» del canon |
| §2, fila 2; §3 | rango de 63 bits | **62 bits** desde el §641 (RFC-0017) | `crates/stark-experiment/src/range_check.rs:8-15`, `:76` |
| §5.3 | el `.wasm`, 1.587.916 B | no se ha vuelto a compilar | — |
| procedencia | «una rama que no se ha fusionado» | fusionada: `71de3d3` y `808e9b9` están en `origin/main` | `git merge-base --is-ancestor` |
| §1 | 14 RFC; `zk-ssl-verify`, 7.443 líneas; `zk-ssl`, 25.970 | eran 13 RFC más la plantilla `0000`; hoy, 19 (0001-0019) más la plantilla; y 10.054 y 28.040 líneas | `git ls-tree spec/rfc`; `wc -l` de los `.rs` de `src/` |

### «Capa 1: no», con el fundamento corregido

El veredicto se sostiene, pero no por las razones de §4 —«por decisión escrita y por tres
medidas»— ni por las «tres decisiones medidas» de la respuesta corta. La decisión escrita es un
comentario (E6) y `VISION.md` es histórico (E9). De los tres hallazgos de `PRINCIPIOS.md` §7, el de
las identidades estaba tachado (E4), el techo de 63 bits es de la configuración sin extensión, que
producción no usa (`crates/zk-ssl/src/lib.rs:221`), y las cifras de demostrable eran de otra
configuración (E7). El fundamento, todo vigente:

1. **El núcleo congelado.** Goldilocks, `Rp64_256` y la firma XMSS no cambian sin versión nueva del
   preámbulo, vectores nuevos y RFC (`spec/NUCLEO.md` §2 y §5; RFC-0005).
2. **El criterio de `CONTRIBUTING.md:149-157`.** No se apila superficie sin un problema presente y
   medido, y «no hay auditoría».
3. **Ningún cuello de capa 1 medido contra el objetivo.** El de `digest_of_proof` se cerró en el
   §209 (E5). El techo que el árbol estima para una cadena de raíces Rescue, ≈1.300 op/s
   (`doc/blueprint-v2.md` §1, ESTIMADO), queda por encima del objetivo RTGS de 21-105 op/s. Y
   cambiar Rescue en los árboles es núcleo (`doc/blueprint-v2.md` §6).

### Lo que dejaron §5 y §8, a `c990868`

| punto | estado | evidencia |
|---|---|---|
| §5.2 y §8.1, la segunda implementación | hecha, en cuatro hitos | la entrada 85 (`BACKLOG.md:720`); §623 y §626 |
| §5.5, `kat_xmss` | hecho | §623 |
| §5.6, el reporte del §575 | enviado | §625 |
| §5.6, `NOTICE`: de ocho y dos a once y tres | corregido | `NOTICE:39-40` |
| §5.6, el asiento que diga que D-F no pesó el PR 293 | pendiente | ningún asiento hasta el §704, ni `BACKLOG.md` ni RFC-0009, lo nombra |
| §5.1 y §8.5, el mapa de una página del módulo | pendiente | solo lo nombra este documento |
| §5.4 y §8.3, el spike y el RFC de capa 2 | pendientes | ninguno de los RFC 0001 a 0019 vuelve a pesar la D-F de RFC-0009 |
| §5.3 y §8.4, `getrandom` y WASM en el issue a RustCrypto | pendiente | `doc/issue-rustcrypto.md` no nombra ninguno de los dos |
| §8.2, la aceptación de `tools/segunda/` | pendiente: el §705 no la cubre | `tools/segunda/README.md` |
| §8.6, «ocho herramientas» | corregido en el §700 | `GENAI.md` §«Cómo se usa, exactamente» |

---

## 1. Lo que la casa tiene, capa por capa ✅

El borrador propone cuatro capas. Se recorren con lo que el árbol tiene en cada una, porque la
pregunta «¿qué integrar verticalmente?» solo se contesta sabiendo qué se posee ya.

| capa del borrador | lo que hay en el árbol | dónde se mide |
|---|---|---|
| **4 · Dominio** | la capa de liquidación (`zk-ssl`, 25.970 líneas), 14 RFC, el paquete de evidencia con doce formas, 380 vectores que jamás se reescriben | `spec/PAQUETE.md`, `spec/rfc/`, `spec/vectors/`, `tools/canon.sh` |
| **3 · Aritmetización** | 35 AIR propias en `stark-experiment`, con sus guardianes: vacuidad por mutación (`mutation.rs`), ranuras y periódicas (`tools/check_constraint_layout.py`), censo de celdas FV-1, spike SMT FV-2, la suite que cuenta lo que una prueba revela (`instrumento_revela.rs`) y los nueve falsadores de la ocultación | `doc/VERIFICACION_FORMAL.md`, RFC-0009 E2 y D-V |
| **2 · Protocolo** | winterfell 0.13.1 y el fork: `Oculta<A>` (140 líneas), `Marca` (69), `LectorAcotado` (84), 350 líneas tocadas en el probador y el verificador; `MerkleConSal` (458) en `zk-ssl-air`; `winter-fri` sin bifurcar. Parámetros: 42 consultas, blowup 16, molienda 21 bits, extensión cuadrática | `crates/winter-*/README.md`, `crates/zk-ssl/src/lib.rs:216-227`, RFC-0009 D-F |
| **1 · Primitivas** | Goldilocks `f64` en producción (68 ficheros; `f128` en 3, experimentos), Rescue Prime `Rp64_256` de `winter-crypto`, Blake3 en el transcript, `xmss 0.1.0-pre.0` clavado con `=`, `chacha20poly1305` y `sha2` de RustCrypto | `range_check.rs:1-5`, `zk-ssl-hash`, `SECURITY.md` |
| **el lado del verificador** (la capa que el borrador no tiene y el nicho exige) | `zk-ssl-verify` (7.443 líneas) sin el probador en su clausura, con compuerta; el núcleo congelado con sus KAT; el kit `arqueo-verify-v0.2.0` con `conformidad.sh` | `spec/NUCLEO.md`, `doc/KIT.md`, `tools/canon.sh` (puerta H2) |

Y una regla escrita en la capa 1, que el borrador invierte: *«escribir criptografía propia aquí sería
un error grave»* (`crates/zk-ssl/Cargo.toml`, sobre el cifrado en reposo). La casa no escribe
primitivas: las elige, las clava con `=`, las declara y las vigila.

**Clausura de dependencias, medida hoy** ✅ (`cargo tree -p <crate> -e normal --prefix none
--locked | sed 's/ (.*//' | sort -u | wc -l`; cuenta paquetes únicos, el propio incluido, con el fork
dentro por el `[patch]`):

| crate | paquetes únicos |
|---|---|
| `zk-ssl-hash` | 17 |
| `zk-ssl-air` | 25 |
| `zk-ssl-verify` | 48 |
| `stark-experiment` | 52 |
| `zk-ssl` | 65 |

---

## 2. El borrador, afirmación por afirmación

| # | afirmación del borrador | veredicto | lo medido o leído |
|---|---|---|---|
| 1 | «Winterfell usaba primariamente `f128`» y el código de ejemplo importa `fields::f128` | **FALSA** para este árbol ✅ | 68 ficheros usan `fields::f64` y 3 `f128`; `Rp64_256` fija Goldilocks y `range_check.rs` cuenta la migración y su hallazgo |
| 2 | Campos de 31 bits (BabyBear, M31) «multiplican la velocidad por 10× a 50×» | **SIN FUENTE** y **CONTRADICE LO MEDIDO** 📖 | `PRINCIPIOS.md` §7.2: *«Goldilocks es demasiado estrecho para identidades»*, colisión en 2³² con 64 bits. Con 31 bits la colisión baja a 2¹⁵ y el range check de 63 bits (`range_check.rs`) quedaría en 30. El multiplicador no trae medida |
| 3 | «Circle STARKs reducen el overhead de memoria hasta un 80 %» | **SIN FUENTE** | ninguna medida, ninguna cita |
| 4 | Poseidon2 dentro del circuito, Blake3 fuera | **PARCIAL** 📖 | Blake3 ya está. Rescue Prime es NÚCLEO (`spec/NUCLEO.md` §2): cambiarlo es versión nueva del preámbulo, 380 vectores y un RFC, no un módulo. ⚠️ Lo que sí está medido es más estrecho: `digest_of_proof` con Rescue cuesta 30,99 ms y es el techo del nodo, 30 → 436 op/s con un hash no algebraico (`doc/DIAGNOSTICO_ESCALADO.md` §0.bis). Es **un** sitio, con su número |
| 5 | «Arqueo debe añadir filas de cegado al final de la tabla para no revelar entradas privadas» | **YA HECHO** 📖 | RFC-0009 D-G, §538: T filas aleatorias, una columna aleatoria, sal en las hojas y cociente aleatorizado; la suite de E2 cuenta cero literales en 100 celdas desde el §538 |
| 6 | Arquitectura `arqueo-math / crypto / air / prover / verifier`, «eliminando dependencias externas no controladas» | **CONTRADICE SU PROPIO TÍTULO** ✅ | es el workspace de winterfell renombrado, y el `Cargo.toml` del borrador depende de `winter-air`, `winter-crypto`, `winter-math`, `winter-prover` y `winter-verifier` 0.13. No integra: envuelve |
| 7 | «Winterfell FRI soporta multi-stage trace commitments y aleatoriedad en tiempo de prueba» | **PARCIAL** 📖 | el tramo auxiliar existe (lo usa `EdadAir`); la aleatoriedad para ocultar no está en upstream: su issue 9 sigue abierto desde 2021 y su README la lista como «Planned» (leído 01-10-2026) |
| 8 | Las cifras de la «simulación»: 1 M de cuentas en 5,40 s, verificador en 4,8 ms, prueba de 136 KB | **SIN MEDICIÓN** ✅ | el «simulador» del borrador hashea datos ficticios y aplica una fórmula; no genera ni verifica una prueba. `CONTRIBUTING.md` §1: *«Cifras sin medición detrás no entran»*. Lo que la casa mide: una prueba de pago oculta pesa entre 145.953 y 167.967 B (`metrics.rs:82-83`) y verificar cuesta 1,5 a 4 ms (`ARQUITECTURA.md`, «Métricas de la capa, medidas») |
| 9 | GPU, CUDA, clústeres: «10 millones de cuentas en segundos» | **YA MEDIDO Y SUSPENDIDO** 📖 | BACKLOG 92, §305 a §318: en una RTX 5090 una prueba dentro del zkVM cuesta 40,9 s más 23,8 s de compresión; *«no hay cruce de curvas»*; suspendida con un disparador cableado en el nodo a 12,4 GiB |
| 10 | Recursión, «Prueba Maestra de menos de 100 KB» | **YA MEDIDO** 📖 | §307: el recibo sucinto mide 223.234 B exactos, constante, 3,3 veces una prueba suelta; agregar compensa a partir de unas cuatro pruebas; `VISION.md` §3.2: winterfell no soporta recursión |
| 11 | «128 bits poscuánticos» con `F_{p^4}` ≈ 2¹²⁴, y en la fila siguiente `F_{p^5}` | **SE CONTRADICE** 📐 y **CONTRADICE LO MEDIDO** 📖 | 2¹²⁴ < 2¹²⁸ en la misma tabla. La casa declara **127 bits conjeturados** y que 128 **demostrables** costarían 125,6 KB por prueba en vez de 36,7 (`lib.rs:211-215`, `FIVE_BACKENDS.md`, `PRINCIPIOS.md` §7.4). El borrador no distingue conjeturado de demostrable, que es la distinción que la casa midió |
| 12 | Las fórmulas de solidez, con el «teorema de proximidad de Jon-Sion» | **SIN FUENTE** | no se localiza ningún resultado con ese nombre; ninguna de las cotas se calcula ni se contrasta con una fuente |
| 13 | «Verificación < 10 ms en CPU y < 300.000 gas en EVM» | **SIN FUENTE** | ningún verificador EVM existe en el árbol ni se mide; la única envoltura sucinta medida costó 23,8 s de GPU (§306) |
| 14 | El DSL `#[arqueo_circuit]` «reduce la curva de 3 semanas a 15 minutos» | **FALSA** como está escrito ✅ | la macro del borrador ignora la estructura anotada y emite siempre el mismo AIR de saldo: no compila nada, lo copia. Lo que la casa tiene en su lugar: 35 AIR a mano con guardianes que nacieron de fallos reales (§39, §50, §66.2) y una espec ejecutable que reproduce el circuito byte a byte (`doc/fv/`) |
| 15 | «Formato APF v1.0» binario, autodocumentado, con Borsh o Protobuf | **YA EXISTE** con otro nombre 📖 | `spec/PAQUETE.md` (doce formas, orden de comprobación, catálogo de rechazos, contrato de salida), `spec/NUCLEO.md` (lo que una segunda implementación reproduce byte a byte), `spec/vectors/nucleo/` (KAT). Un segundo formato para lo mismo es «dos definiciones», lo que §254 y §255 prohíben |
| 16 | «Proof of Reserves 2.0»: activos ≥ pasivos, Basilea III, DeFi, L2 | **CONTRADICE EL ALCANCE DECLARADO** 📖 | `README.md`: *«Prueba conservación, no solvencia: las pruebas hablan del libro, no del mundo»*. `VISION.md` P4: *«Ampliar el alcance no es un bien en sí»*. `doc/ESCALADO.md`, cabecera: ampliar el alcance es decisión de misión por VISION §5, no de backlog. `ROADMAP-ECOSISTEMA.md`: la casa compite en liquidación e ISO 20022, *«no en el nicho de Ethereum»* |
| 17 | Bindings C-FFI, Java, Python; WASM «< 1 MB» | **PARCIAL** ✅ | nada de eso existe. Pero el verificador **compila hoy** a `wasm32-wasip1` (§5.3); el tamaño no se ha medido |
| 18 | «Verificación formal al 100 % del verifier core y 0 vulnerabilidades en AIR» | **CONTRADICE LO MEDIDO** 📖 | `doc/VERIFICACION_FORMAL.md` §0: la suite prueba puntos, no el universal; FV-1 es un censo sintáctico; FV-2 un spike SMT sobre `circuit_refund`; FV-3 «el horizonte, nombrado sin prometerlo» |
| 19 | Donación a una fundación, «Consorcio Arqueo», comité técnico, que las Big Four acepten APF como evidencia legal | **SIN FUENTE** y fuera de los principios 📖 | `PRINCIPIOS.md` §8 retiró una hoja de ruta más modesta que esta porque *«decirlo sería faltar al principio de transparencia»*; §10: *«Falta descentralización, auditoría externa y adopción. Ninguna de las tres es un problema criptográfico»*. `VISION.md` P8: la legitimidad no depende del autor ni de un sello |
| 20 | Un módulo propio de molienda (`arqueo-pow`) con Blake3 | **DUPLICA** lo que ya se usa ✅ | `ProofOptions` lleva `grinding_factor` (`winter-air/src/options.rs:98`) y la casa lo fija en 21 bits (`lib.rs:220`). Una molienda fuera del transcript del probador no ata nada: tiene que vivir dentro de la prueba |
| 21 | Un `F_{p^4}` propio sobre BabyBear, con Frobenius y norma | **INNECESARIO** 📖 | winterfell trae extensión cuadrática y cúbica y la casa usa la cuadrática sobre Goldilocks (`lib.rs:221`); el código del borrador no lleva tests ni vectores y usa `%` sobre `u64`, que no es tiempo constante |

Lo que **sí** acierta el borrador, dicho sin regatear: que el verificador tiene que vivir fuera del
probador, que la especificación tiene que estar desacoplada del código, que hacen falta segundas
implementaciones y vectores. Las cuatro cosas están en `ROADMAP-ECOSISTEMA.md` desde agosto, las
tres primeras selladas (§197 a §199) y la cuarta es la entrada 85 del BACKLOG.

---

## 3. El código del borrador, leído contra el árbol ✅

No se compila: se lee contra el winterfell vendorizado, que es el que el árbol usa.

- `use winter_prover::{DefaultTraceLwe, ...}`: **cero apariciones** en `crates/winter-prover/src`.
  El tipo es `DefaultTraceLde` (`trace/mod.rs:13`).
- `trace.fill(|execution_trace| { execution_trace[0][0] = ... }, |buf| { ... })`: la firma real es
  `fill<I, U>(init: I, update: U)` con `I: FnOnce(&mut [B])` y `U: FnMut(usize, &mut [B])`
  (`trace/trace_table.rs:164-168`). El primer cierre recibe **una fila**, no una matriz; el segundo
  recibe **dos** argumentos. Ninguno de los dos cierres del borrador tipa. Y `buf[0] = buf[0]` es un
  no-op que deja la columna del saldo con el valor de la fila anterior solo por accidente.
- El AIR del borrador impone `S_{t+1} = S_t + Δ_t` y nada más. Sin comprobación de rango sobre
  `Δ_t`, un `Δ` «negativo» es un elemento grande del campo y el saldo da la vuelta: es la clase de
  fallo que `ARQUITECTURA.md` cerró bajo «Ninguna vía para crear dinero» y que `range_check.rs` y
  `double_entry.rs` protegen con 63 bits y partida doble. El borrador lo deja para una «Fase 2».
- La macro `#[arqueo_circuit]` no lee `#[trace_col]`, `#[public_input]`, `#[transition_logic]` ni
  `#[boundary_constraints]`: emite el mismo AIR sea cual sea la estructura. Un DSL que no compila la
  entrada no es un DSL.
- El «simulador» no construye ninguna prueba: encadena hashes de ceros y multiplica enteros. Sus
  milisegundos miden ese bucle.

Veredicto de esta sección: **ninguna línea del borrador es base de nada**. Lo que un módulo de la
casa necesita de código ya está escrito, con tests, en los crates de §1.

---

## 4. Integración vertical: qué significa para Arqueo, capa por capa

Integrar verticalmente es controlar lo que, si cambia, cambia qué es dinero válido o qué acepta un
verificador. Con ese criterio:

**Capas 4 y 3, y el lado del verificador: ya son de la casa, y ahí está el estándar.** Un estándar
de evidencia no es un probador: es **lo que un tercero reproduce sin fiarse del autor**. Eso es
`NUCLEO.md` más `PAQUETE.md` más los vectores más el kit. HBS-STATE y `mtc-core` salieron de aquí
con esa forma —especificación, implementación de referencia, vectores, un juez— y es la forma que
el nicho de Arqueo admite. No hace falta otro formato ni otro nombre.

**Capa 2: de la casa en la medida exacta del fork, y es una decisión abierta.** Lo medido en
sesión el 01-10-2026 📖: winterfell no tiene un commit en `main` desde el 19-07-2025; su PR 293 de
conocimiento cero lleva abierto desde julio de 2024, con fallos reportados en enero de 2026 y sin
respuesta; su autor, de Miden, fusionó el 02-01-2026 la migración de la máquina virtual de Miden a
Plonky3; en crates.io dependen de `winter-prover` siete crates, ninguno grande. La cláusula de
reversibilidad de RFC-0009 D-F —«si winterfell publica ocultación propia, se sigue a winterfell y
el fork se retira»— tiene hoy un disparador improbable. Y lo que la casa descartó en D-F por coste,
Plonky3, se midió hoy ✅ con un crate de prueba fuera del árbol (`p3-uni-stark`, `p3-fri`,
`p3-merkle-tree`, `p3-commit`, `p3-challenger`, `p3-dft`, `p3-field`, `p3-matrix`, `p3-goldilocks`,
`p3-symmetric`, `p3-blake3`, `p3-air`, todos 0.8.0):

| medida | valor |
|---|---|
| clausura de dependencias (mismo método que §1) | **54** paquetes, frente a 52 de `stark-experiment` |
| ocultación en `p3-fri` 0.8.0 | `HidingFriPcs`, `HidingFriPcsProverData` |
| ocultación en `p3-merkle-tree` 0.8.0 | `MerkleTreeHidingMmcs` |
| ocultación en `p3-uni-stark` 0.8.0 | `is_zk` en 44 sitios, `randomized` en 9 |
| lookups en `p3-uni-stark` 0.8.0 | `lookup` en 24 sitios |

⚠️ Lo que esta medida **no** dice: si el STARK univariante de Plonky3 cubre el tramo auxiliar que
`EdadAir` necesita (D-F decía que no en la versión de entonces; en 0.8 hay lookups, y si bastan no
se ha medido), ni cuánto cuesta reabrir 35 AIR (D-F: 10.100 líneas de implementación y 4.341 de
transición), ni que Plonky3 falle cerrado con una prueba malformada: su README advierte que *«the
verifier may panic on certain malformed proofs»* y pide `catch_unwind`, que es exactamente lo que
el §575 arregló en el fork con un lector acotado. Migrar no quita esa deuda: la traslada.

**Capa 1: no, por decisión escrita y por tres medidas.** La regla del `Cargo.toml` de la capa; P6
de `VISION.md`, *«coherencia sobre brillo»*: una optimización que añade superficie de fallo difícil
de auditar se descarta aunque mejore latencia o tamaño; y los tres hallazgos de `PRINCIPIOS.md` §7
sobre el campo (identidades, techo de 63 bits, conjeturado frente a demostrable), que un campo de
31 bits empeora en los tres. La única decisión de capa 1 con un número detrás es la del hash en
**un** camino caliente del nodo (§4 de la tabla), y es un cambio del LIBRO, no del núcleo.

---

## 5. Lo que sí sale de las piezas de la casa, con su medida

### 5.1 Nombrar el módulo que ya existe, sin crear un crate

El «módulo criptográfico Arqueo» es el conjunto que hoy no tiene nombre pero sí contrato:
`zk-ssl-hash` (las primitivas de formato, una definición), `zk-ssl-air` (los AIR que un tercero
juzga y `MerkleConSal`), `zk-ssl-verify` (el verificador sin probador), los tres ficheros nuevos del
fork (`oculta.rs`, `marca.rs`, `acotado.rs`), la foto del probador prístino (`kat_probador.rs`), los
falsadores y `mutation.rs`. Su contrato es `spec/NUCLEO.md`. Un crate nuevo sin llamador va contra
el §580 (*«la casa no cría código sin llamador»*); lo que falta es un mapa de una página que lo
nombre, y entra por un asiento.

### 5.2 La segunda implementación: la entrada 85, hecha en su primer hito ✅

BACKLOG 85 lo dice desde antes de este borrador: *«una spec que sólo implementa un código no es
una spec, es documentación»*, y pone el listón: **un verificador en otro lenguaje que pase los
vectores**. El primer hito medible es el núcleo congelado, y está hecho: `tools/segunda/nucleo.py`,
Python sin dependencias, escrito desde `NUCLEO.md` §6 y de ningún `.rs` de la casa, con
`juez_nucleo.py` como único llamador.

```text
python3 tools/segunda/juez_nucleo.py
  autotest de las primitivas (BLAKE3 contra blake3 1.8.5; Rescue contra el vector Sage de winter-crypto): VERDE
  OK   acuse_digest … OK   recibo_digest            (26 líneas)
  nucleo: 26 de 26 vectores reproducidos byte a byte
```

Lo que destapó, que es para lo que la entrada existe (detalle en `tools/segunda/README.md`):

| hueco | dónde | medido |
|---|---|---|
| `hoja_de_acuse(hashPrueba, seq, n)` no estaba en §6; `RPC.md` la da con `epoca` en «`zkssl_ackPath`» y dice `epoca = logSeq + 1` en «El acuse en la respuesta (§274)», pero nadie dice que el `seq` de la hoja es ese `logSeq` | `NUCLEO.md` §6, `PAQUETE.md` §4 paso 3/3 | con `seq` no reproduce; con `seq + 1`, sí |
| `hash_del_lote`: la longitud codificada es la de los bytes, `48·k`, no `k` | `NUCLEO.md` §6, RFC-0014 E2 | con `k` no reproduce; con `48·k`, sí |
| `cima`: no dice si cada hoja pasa por `mmr_hoja` dentro | `NUCLEO.md` §6 | pasa, lectura RFC 6962 |
| los KAT codifican el u64 de dos maneras, ocho bytes LE o número hex, según el campo | `spec/vectors/nucleo/` | un juez lo sabe por fuera |

Ninguno cambia un byte del formato: cambian la **prosa** de la spec, que es lo que una segunda
implementación mide, y las tres precisiones ya están en `NUCLEO.md` §6 con el error registrado.

**Segundo hito, hecho en la misma sesión** ✅: `tools/segunda/kat_xmss/xmss.py`, la verificación XMSS^MT de
RFC 8391 escrita desde el RFC, y `juez_cabezas.py`, que recorre los 331 ficheros con cabezas de
`spec/vectors/`:

```text
python3 tools/segunda/juez_cabezas.py
  cabezas: 376 de 418 verifican; cofirmas: 7 de 11; ficheros con cabezas: 323;
  con alguna que NO verifica: 46, de ellos positivos segun su MANIFIESTO: 0
  FALSADOR: un byte volteado en la firma y otro en el preambulo NO verifican: VERDE
```

Los 42 que no verifican son los que los negativos fabrican, y cada uno cae por la regla que el
manifiesto le asigna. Con esto la segunda implementación cubre los pasos 1/3 y 2/3 del paquete
—la cabeza recompone, la firma verifica, el preámbulo es el esperado, el índice embebido queda por
debajo del declarado— y las cofirmas de testigo.

**Tercer hito, hecho** ✅: `tools/segunda/verificador.py`, el segundo verificador, con el contrato
del mando y las cinco formas sin STARK, juzgado por `tools/conformidad.sh` con los manifiestos
del binario: `paquete` 70 de 70, `consumo` 14 de 14, `conflicto` 16 de 16, `ancla` 21 de 21, y
salida idéntica línea a línea a la del binario en todos los positivos. Con esto la conformidad de
esas cuatro familias deja de ser autoconformidad: dos códigos, uno en Rust y otro en Python
escrito desde la spec, dicen lo mismo entrada a entrada. Los jueces entran en el canon como bloque propio
(`tools/canon.sh`, «3 duodecies»), a pin cero: lo que no corre no protege. El `--sello` entero,
corrido con el bloque dentro: **VERDE**, 18 crates en sus pines, 1.054 s de tests, las nueve
herramientas, los diez manifiestos y el artefacto reproducible.

### 5.3 El verificador en WebAssembly: verifica los 316 vectores bajo wasmtime ✅

```text
cargo build -p zk-ssl-verify --release --target wasm32-wasip1 --locked   -> 1.587.916 B
tools/conformidad.sh con tools/segunda/wasm_runner.py (wasmtime 49.0.0), diez manifiestos:
  ancla 21/21 · completitud 73/73 · conflicto 16/16 · consumo 14/14 · edad 11/11 · pago 9/9 ·
  paquete 70/70 · pendiente 9/9 · prenda 9/9 · rechazo 84/84          -> 316 de 316, como el nativo
```

| medida | nativo | `wasm32-wasip1` |
|---|---|---|
| tamaño | 2.517.640 B | 1.587.916 B |
| los diez manifiestos, de pared | 9,28 s | 32,78 s |
| un vector, media de cinco | 18,4 ms | 93,3 ms, 66,8 de ellos arrancar Python y wasmtime |

Sin tocar una línea del verificador. En `wasm32-unknown-unknown`, el objetivo del navegador, lo
para el generador de números aleatorios que `xmss` arrastra por `rand 0.10` para generar claves,
que el verificador no usa: una línea más para el issue a RustCrypto de la entrada 77. ⚠️ Lo que
**no** se ha medido: el `.wasm` optimizado para tamaño ni `wasm-opt`; y el borrador prometía
«< 1 MB» sin medir, y mide 1,5 MiB tal cual sale. El roadmap histórico marcaba WASM como
«evaluar», pero para el probador (`ROADMAP-ECOSISTEMA.md:34`); el verificador es el que tiene
sentido, porque es lo que un tercero corre.

### 5.4 La capa 2 por RFC, con las cifras nuevas

D-F pesó dos caminos y no vio el tercero (el PR 293), y la premisa de su reversibilidad ha cambiado.
Lo honesto es volver a pesar con el mismo método, no decidir aquí: un spike como el SPIKE-B-P4 que
mida (a) si `p3-uni-stark` 0.8 cubre `EdadAir`, (b) líneas de reapertura reales de los 35 AIR,
(c) la foto de D-R contra Plonky3 apagado y la suite de E2 encendido, (d) un lector acotado o
`catch_unwind` en el verificador, y (e) si upstream winterfell sigue mudo. Con eso, un RFC decide
entre asumir el fork con nombre propio o migrar, y pone fecha. Las dos salidas cumplen VISION §5 si
se declaran; la que no lo cumple es seguir con una cláusula de salida que nadie vigila.

### 5.5 Modularizar como `hbs-state` y `mtc-core`: decidido, pieza a pieza

El criterio es el de `doc/MTC.md`: sale lo que comparte con Arqueo la infraestructura y la
disciplina, **no el problema**, y tiene consumidores fuera.

| pieza | ¿sale? | por qué |
|---|---|---|
| el fork de winterfell | **todavía no** | su salida depende de la decisión de capa 2 (§5.4): extraerlo antes de medir el spike es decidir sin medir. Si el RFC dice «asumir el fork», sale con nombre propio y `NOTICE` cambia antes |
| `spec/` y los vectores | **no** | son el problema, no la infraestructura; y el doble hilo del canon (`check_nucleo`, el OpenRPC generado, los vectores de los bancos) vive de tenerlos al lado del código. Lo que hace estándar a una spec no es un repositorio aparte: son dos implementaciones y un arnés, y eso es lo que esta rama construye. La forma distribuible ya existe: el kit |
| las herramientas de higiene de AIR | **más adelante** | valen para cualquier AIR de winterfell, pero la parte Python está atada a las convenciones de la casa, y el ecosistema winterfell se está quedando sin consumidores (§4) |
| **`tools/segunda/kat_xmss/` con `xmss.py`** | **sí, y está listo** ✅ | verificar XMSS^MT de RFC 8391 no es el problema de Arqueo y lo necesita cualquiera que firme con RFC 8391, `hbs-state` incluido. Cero dependencias, cero bytes del protocolo. Cierra el «KAT ausente» de la entrada 77: un corpus producido por el crate clavado y verificado por otra implementación, 8 de 8 y tres falsadores callados. Partido con `git subtree split` y empujado como rama `xmss-kat-main` (§623); hace de repositorio por sí solo |

El resto de `tools/segunda/` —`nucleo.py`, los jueces del núcleo y de las cabezas— **no** sale: es
la segunda implementación del protocolo y su sitio es al lado de la spec y de los vectores que
juzga, en el canon.

### 5.6 La deuda del §575 y el registro

Fuera de este documento pero antes que todo lo anterior: el reporte privado a winterfell del aborto
por reserva de memoria (§575, *«avisarle es deuda»*), por el canal que su política pide; y el
asiento que diga que D-F no pesó el PR 293 y que `NOTICE` cuenta ocho ficheros tocados y dos nuevos
cuando son once y tres desde el §537 y el §575.

---

## 6. Veredicto

| pregunta | respuesta |
|---|---|
| ¿Integrar verticalmente? | Ya está, en las capas que importan para el nicho. Capa 2: decidir por RFC. Capa 1: no |
| ¿Un módulo criptográfico nuevo? | No: nombrar el que existe. Un crate sin llamador va contra el §580 |
| ¿Coger lo útil de winterfell? | Ya se cogió: 350 líneas y tres ficheros, con la foto que prueba que apagado es winterfell byte a byte |
| ¿Y del ecosistema? | Plonky3 0.8 lleva ocultación y lookups con una clausura de 54 paquetes, medida; si cubre `EdadAir` es lo primero que hay que medir |
| ¿Ser el estándar del nicho? | Con la entrada 85: una segunda implementación que pase los vectores. Cuatro hitos hechos y en el canon: 26 de 26 KAT del núcleo, las cabezas firmadas de todos los vectores con XMSS^MT desde RFC 8391, el segundo verificador con 121 de 121 entradas de las cuatro familias sin STARK, y el verificador STARK en Python sobre las pruebas ocultas y con sal de las seis familias que las llevan, 23 de 23 pares con el veredicto y la causa del binario (§626). Lo que sigue siendo promesa: las cinco AIR, transcritas del único sitio donde existen, y el mando entero de esas formas en el segundo código |
| ¿El borrador? | Ninguna línea sirve de base; cuatro de sus ideas ya están selladas y una contradice el alcance declarado |
| ¿Modularizar como `hbs-state` y `mtc-core`? | Una pieza, hoy: `kat_xmss` con `xmss.py`, lista para `git subtree split`. El fork, cuando el RFC de capa 2 decida; la spec, nunca por ese criterio |

---

## 7. Lo que este documento NO afirma

- Que Plonky3 sea mejor que el fork: solo que su coste ya no es el que D-F pesó y que se mide antes
  de decidir.
- Que el verificador funcione en un **navegador**: funciona bajo wasmtime con WASI, 316 de 316.
  El objetivo sin sistema lo para `getrandom` vía `xmss`, y no se ha medido más allá.
- Que un campo de 31 bits sea más lento o más rápido: que agrava tres hallazgos medidos sobre
  identidades y rango, y que el multiplicador del borrador no trae medida.
- Nada sobre solvencia, reservas ni Basilea III: el README dice que las pruebas hablan del libro, no
  del mundo, y este documento no lo mueve.
- Que las cifras del borrador sean falsas: que **no están medidas**, que es lo único que la casa
  necesita saber para no copiarlas.
- Que la segunda implementación sea una segunda implementación del **protocolo**: lo es del
  núcleo congelado, de la cabeza firmada con sus cofirmas, de las cinco formas sin STARK y, desde
  el §626, de la verificación STARK de las seis familias con prueba. No lo es de sus cinco AIR,
  transcritas del `.rs` porque ninguna RFC escribe sus restricciones, ni del mando de esas formas,
  que en el segundo código todavía no se compone con la cabeza y la firma.

---

## 8. Pendientes que deja

1. ✅ **Hecho en el §626.** La entrada 85, cuarto hito: un verificador STARK de winterfell en Python
   para las formas con prueba (FRI, DEEP y las AIR como jueces). ⚠️ **Corrección (§626):** aquí ponía
   «los 23 AIR», y es inexacto: el paquete verifica **cinco** —Banda, Edad, Prenda, CobroPendiente y
   PagoEnCurso—; los 23 de `stark-experiment` son del probador y no viaja ninguno en un vector.
   **Decidido el lenguaje, con medida**: Python,
   no un lenguaje compilado. Lo medido en esta máquina sobre las primitivas ya escritas —permutación
   Rescue 1,28 ms, un merge Blake3 78 µs, una multiplicación en la extensión cuadrática 0,65 µs—
   proyecta ~0,73 s por prueba y ~59 s por el catálogo entero (81 pruebas STARK), del orden de lo
   que ya tardan suites del árbol. Un segundo verificador compilado iría 10× a 50× más rápido y no
   hace falta: la segunda implementación no se mide por su velocidad sino por decir los mismos
   bytes, y meterlo en otro lenguaje añadiría una caja de compilación por una conformidad que no la
   necesita. Se queda en Python, con la permutación y Blake3 que ya tiene. El hito es escribir FRI,
   el muestreo DEEP y las AIR como restricciones; el coste es de concepto, no de rendimiento.
   Medido al hacerlo: 0,2 a 0,5 s por prueba, por debajo de los 0,73 proyectados; el juez entero, 12 s.
2. El asiento del autor que acepte o revierta lo que esta rama deja: `tools/segunda/` con `kat_xmss/`,
   el bloque «3 duodecies» del canon y las precisiones de prosa de `NUCLEO.md` §6 y §8; y, si lo
   quiere fuera, el `git subtree split` de `kat_xmss/`.
3. El spike de capa 2 (§5.4), antes de ningún RFC.
4. `wasm32-unknown-unknown`: una línea más para el issue a RustCrypto (entrada 77), `xmss`
   arrastra `rand` al verificador; y medir el `.wasm` optimizado para tamaño.
5. El mapa de una página del módulo (§5.1), como asiento, sin crate.
6. La cifra «ocho herramientas de `tools/`» de `README.md`, `README_EN.md`, `GENAI.md` y
   `RESUMEN_EJECUTIVO.md` ya no cuadra con el bucle del canon, que corre nueve; no se toca aquí
   porque el bloque nuevo no es una de ellas, y se deja dicho.

---

## Fuentes

- Este árbol en `e8ac246`: `README.md`, `PRINCIPIOS.md` §6 a §10, `CONTRIBUTING.md` §1,
  `ARQUITECTURA.md` («Métricas de la capa, medidas», «Ninguna vía para crear dinero»),
  `SECURITY.md` §3.7, `spec/NUCLEO.md` §1 y §2, `spec/PAQUETE.md`, `spec/rfc/0009-*.md` (D-F, D-G,
  D-I, D-R, D-V), `doc/VERIFICACION_FORMAL.md` §0, `doc/DIAGNOSTICO_ESCALADO.md` §0.bis,
  `doc/ESCALADO.md` (cabecera), `doc/historia/VISION.md` (P4, P6, P8, §3.2, §5),
  `doc/historia/ROADMAP-ECOSISTEMA.md`, `BACKLOG.md` 77, 85, 92, `AUDITORIA.md` §533, §575,
  `crates/zk-ssl/src/lib.rs:211-227`, `crates/zk-ssl/src/metrics.rs:82-83`,
  `crates/stark-experiment/src/range_check.rs:1-25`, `crates/winter-prover/src/trace/mod.rs:13`,
  `crates/winter-prover/src/trace/trace_table.rs:164-168`, `crates/winter-air/src/options.rs:98`,
  `crates/zk-ssl/Cargo.toml` (el comentario sobre criptografía propia).
- GitHub y crates.io, leídos el 01-10-2026: `facebook/winterfell` (commits de `main`, issue 9,
  PR 293, PR 383, política de seguridad), `0xMiden/miden-vm` (README, `Cargo.toml`, PR 2472),
  `Plonky3/Plonky3` (README, CHANGELOG), `crates.io/api/v1/crates/winter-prover` y sus dependencias
  inversas.
- Fuentes de Plonky3 0.8.0 descargadas por `cargo fetch` en el crate de prueba y leídas con `grep`:
  `p3-fri`, `p3-merkle-tree`, `p3-uni-stark`.
- `tools/segunda/`: `nucleo.py`, `juez_nucleo.py`, `wasm_runner.py` y su `README.md`, con las salidas
  de los dos spikes; `winter-crypto 0.13.1`, `src/hash/rescue/rp64_256/{mod,tests}.rs` (la
  permutación y su vector Sage); `blake3 1.8.5` del `Cargo.lock` (las 19 salidas de referencia);
  `wasmtime` 49.0.0 de PyPI.
- El borrador aportado por el autor, en su integridad, como objeto de la evaluación y no como fuente.
