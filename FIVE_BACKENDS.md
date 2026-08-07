# Cinco backends criptográficos: comparativa con métricas reales

Todos los números se midieron ejecutando el **mismo circuito de
cumplimiento**, en la misma máquina (WSL2 sobre un portátil de consumo) y
**todos en modo release**. No hay cifras citadas de la literatura.

Reproducir:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
cargo test -p nova-experiment --release --features test-setup -- --nocapture
```

---

## Nota metodológica: un error corregido

Una versión previa comparaba cifras de Groth16 y Halo2 medidas en
**debug** con cifras de STARK medidas en **release**. Al homogeneizar, los
dos primeros resultaron entre **11 y 14 veces más rápidos** de lo que se
les atribuía. Se documenta el error en vez de corregirlo en silencio.

---

## Tabla comparativa (circuito de cumplimiento)

| | Groth16 | Halo2 / IPA | STARK / FRI | PLONK / KZG |
|---|---|---|---|---|
| **Paradigma** | R1CS | Plonkish | AIR | Plonkish |
| **Curva / campo** | BLS12-381 | Pallas | Goldilocks | BLS12-381 |
| **Trusted setup** | Por circuito | No | No | **Universal** |
| **Setup / SRS** | 438 ms | 16,3 s | ninguno | 26,3 s (reutilizable) |
| **Compilación por circuito** | (incluida arriba) | — | — | 12,8 s |
| **Generación** | **0,42 s** | 4,86 s | **0,039 s** | 6,85 s |
| **Verificación** | 5 ms | 91 ms | **1 ms** | 8 ms |
| **Tamaño de prueba** | **192 bytes** | 4.096 bytes | 36,7 KB | 1.008 bytes |
| **Resistencia cuántica** | No | No | **Sí** | No |
| **Profundidad del árbol** | 20 | 20 | 32 | 20 |

## Circuito de partida doble

| | Groth16 | STARK | PLONK / KZG |
|---|---|---|---|
| Tamaño | 27.562 restricciones | 113 restricciones × 41 columnas × 1024 filas | 84.801 puertas |
| Setup / SRS | 1,12 s | ninguno | 54,7 s |
| Compilación | — | — | 48,5 s |
| Generación | **1,17 s** | — | 26,4 s |
| Verificación | 5 ms | — | 10 ms |
| Tamaño de prueba | **192 bytes** | — | 1.008 bytes |

(Halo2 tiene el circuito de partida doble implementado y verificado con
`MockProver`, pero no medido con pruebas reales.)

---

## Nova: un paradigma distinto, medido aparte

Los cuatro backends anteriores producen una **prueba monolítica** del
circuito entero. Nova hace algo diferente: **pliega** una secuencia de
pasos y comprime al final. No compite en el mismo eje, así que sus
números van aparte.

| Fase | Coste medido |
|---|---|
| `PublicParams::setup` (una vez) | 4,02 s |
| **Por transacción (`prove_step`)** | **~250 ms, CONSTANTE** |
| `RecursiveSNARK::verify` | 108 ms |
| `CompressedSNARK::prove` (cierre) | 1,84 s |
| `CompressedSNARK::verify` | 50 ms |
| Restricciones por paso | 10.764 (primaria) / 10.538 (secundaria) |

**El dato que justifica el backend**: el paso 9 costó **0,77 veces** el
paso 1. El coste no crece con la longitud de la cadena — que es
exactamente la propiedad que define al plegado y que ninguno de los otros
cuatro tiene.

El perfil resultante —barato durante el día, caro al cerrar— coincide con
el de una cámara de compensación que liquida durante la jornada y cierra
por la noche.

### ⚠️ Matices honestos

**El sobrecoste del plegado es sustancial.** 10.764 restricciones por paso
para un circuito que hace **un solo hash Poseidon**. Casi todo es el
"circuito verificador" que Nova inserta en cada paso. Nova no es gratis:
cobra maquinaria fija y solo se amortiza con muchos pasos.

**Los ~250 ms NO son comparables** con el 1,17 s de la partida doble en
Groth16. El paso medido es trivial; un paso con partida doble real
rondaría las 38.000 restricciones. Lo demostrado es que *el mecanismo
funciona y su coste es constante*, no que Nova sea más rápido aquí.

**Alcance**: `nova-experiment` se cerró en nivel de prueba de concepto.
No implementa el circuito de cumplimiento ni la partida doble, a
diferencia de los otros cuatro.

---

## Los hallazgos que solo aparecen midiendo

### 1. PLONK-KZG es el generador más lento de los cuatro

Es el resultado más contraintuitivo del proyecto. PLONK-KZG suele
presentarse como el estándar de la industria, y aquí resulta **16 veces
más lento que Groth16** en el circuito de cumplimiento y **22 veces más
lento** en el de partida doble.

**Matiz importante y honesto**: parte de esa diferencia puede deberse a
la *implementación* (`dusk-plonk` frente a arkworks, muy optimizado) y no
al esquema en sí. Estos datos no permiten separar ambos efectos. La
afirmación defendible es: *esta implementación de PLONK-KZG es la más
lenta de las cuatro medidas*, no *PLONK-KZG es lento*.

Contribuye también que su hash sea más caro: **997 puertas** por hash
Poseidon de aridad 2, frente a ~300 restricciones en Groth16.

### 2. Pero su setup es el único verdaderamente reutilizable

Groth16 exige una ceremonia **por cada circuito**. Este proyecto ya tiene
dos (solvencia y partida doble), así que serían dos ceremonias.

PLONK-KZG usa un SRS **universal**: una sola ceremonia sirve para todos
los circuitos presentes y futuros. La compilación por circuito (12,8 s /
48,5 s) es determinista y **sin secretos** — no es una segunda ceremonia.

Y existe una ceremonia pública real sobre BLS12-381, coordinada por Dusk,
con herramienta de conversión al formato `PublicParameters`. Es la única
de las cuatro opciones donde hay una ceremonia **ya celebrada** y no solo
un mecanismo implementado.

### 3. Sin extensión de campo, un STARK sobre Goldilocks tiene un techo de 63 bits

El campo mide 64 bits y la solidez no puede superarlo por muchas queries
que se añadan. La configuración "rápida y compacta" que uno elegiría por
defecto **no es comparable** con los ~128 bits de los otros tres.

### 4. La brecha entre seguridad conjeturada y demostrable en STARK

| Configuración | Tamaño | Generación | Conjeturada | Demostrable |
|---|---|---|---|---|
| blowup 8, sin extensión | 27,7 KB | 25 ms | 63 bits | 26 / 24 |
| blowup 8, ext. cuadrática | 32,7 KB | 28 ms | 95 bits | 26 / 47 |
| **blowup 16, ext. cuadrática** | **36,7 KB** | **39 ms** | **127 bits** | 29 / 63 |
| 120 queries, grinding 20, ext. cúbica | 125,6 KB | 48 ms | 128 bits | **128 / 128** |

Los 127 bits conjeturados que suele citar el ecosistema conviven con
29-63 demostrables. Cerrar la brecha cuesta 125,6 KB en vez de 36,7 — y
aun así sigue siendo el generador más rápido.

### 5. Solo dos de seis librerías se defienden del uso inseguro

Este proyecto ha documentado a mano, en cada backend, que generar
parámetros con un `setup()` local no vale para producción. Groth16,
Halo2, STARK y PLONK-KZG lo permiten y confían en que quien lo use lo
advierta.

**`nova-snark` lo impide en código.** Su mensaje de error es literal:

```text
HyperKZG::setup is disabled in production builds. Use
PublicParams::setup_with_ptau_dir ... with ptau files from a trusted
setup ceremony. For tests, enable the 'test-utils' feature.
```

Y ofrece la vía correcta resuelta: `setup_with_ptau_dir` consume ficheros
ptau de **Perpetual Powers of Tau**, la ceremonia pública existente.

En `crates/nova-experiment` la feature `test-setup` se expone
deliberadamente en la línea de comandos, y no oculta en
`dev-dependencies`, para que quien ejecute los tests vea que está
habilitando algo que no vale para producción.

**Y hay una segunda.** Al evaluar `risc0-zkvm` (ver hallazgo 7) aparece
el mismo patrón por otra vía: su tipo de recibo incluye una variante
`Fake`, *"sin integridad criptográfica, usada solo para desarrollo"*, y
ofrece la feature `disable-dev-mode` descrita como *"desactiva el modo de
desarrollo para que probar y verificar no puedan falsearse. Usado para
evitar que un `RISC0_DEV_MODE` mal puesto rompa la seguridad en sistemas
de producción"*.

Nombra **el escenario concreto** —una variable de entorno olvidada— que
es exactamente cómo ocurren estos fallos en la práctica.

| Librería | Ante pruebas sin garantía criptográfica |
|---|---|
| `nova-snark` | **Lo impide** en compilaciones de producción |
| `risc0-zkvm` | Lo permite, pero da una feature para bloquearlo |
| Las otras cuatro | Lo permiten en silencio |

**Dos de seis se defienden activamente del uso inseguro.** Es un eje que
ninguna tabla de rendimiento captura, y distingue una librería pensada
para producción de una pensada para publicar un paper.

### 6. AIR carece de restricciones de copia, y eso obliga a rediseñar

Al portar la partida doble a STARK apareció un agujero de solidez que **no
existe en los otros tres paradigmas**: nada obliga a que las dos subidas
del árbol (hoja antigua y hoja nueva) usen los mismos hermanos. Un
probador podría usar hermanos distintos en cada una y fabricar una raíz
que no corresponde a la misma posición del árbol — y los testigos
honestos nunca lo revelarían.

La solución fue un diseño en **lockstep** (dos carriles avanzando nivel a
nivel con el hermano forzado a ser idéntico), verificado de forma aislada
en `stark-experiment::dual_climb`.

En PLONK y Halo2, ambos Plonkish, basta con reutilizar el mismo
`Witness`: las restricciones de copia lo garantizan. En Groth16, igual.

**Conclusión práctica: portar un circuito de Plonkish a AIR no es
mecánico, ni siquiera cuando la lógica es idéntica.**

---

### 7. Un zkVM no es comparable en igualdad de condiciones, y eso ya es un dato

Se evaluó **RISC Zero** como sexto paradigma. Su planteamiento es opuesto
al de los otros cinco: en vez de escribir circuitos a mano, se compila un
programa Rust normal y el zkVM prueba su ejecución.

El encaje conceptual era bueno: usa **STARK sobre Goldilocks**, el mismo
sistema y el mismo campo que el backend elegido, así que la comparación
habría sido casi un experimento controlado —única variable: cómo se
escribe la lógica—.

Y la vía sin ceremonia existe: `InnerReceipt::Succinct` prueba *"cómputos
de zkVM arbitrariamente largos con un único STARK"*. El envoltorio
Groth16, que sí exige ceremonia, solo hace falta para verificar en
cadena.

**Pero no cumple el criterio metodológico del proyecto.**

Los cinco backends se instalan con `cargo add` y nada más. Ese criterio
descartó `dusk-plonk 0.21` (exigía nightly), `halo2-lib` (dependencias
git sin fijar) y `plonk-core` (sin publicar).

RISC Zero necesita una **toolchain externa** para compilar el programa
invitado a RISC-V: su tabla de features indica que `prove` está
disponible en *"todos los objetivos excepto rv32im"*, y el README remite
a `cargo risczero`, un instalador aparte.

**No es un defecto**: es el precio inevitable de compilar programas
arbitrarios. Pero medirlo junto a los otros cinco falsearía la
comparación, así que **se documenta en vez de implementarse**.

Cifras que sí quedan registradas de la evaluación:

| | Backend STARK propio | RISC Zero |
|---|---|---|
| Dependencias | **3** | **349** |
| Instalación | `cargo add` | `cargo add` + toolchain externa |
| Seguridad declarada | 127 bits conjeturados | 98 bits conjeturados |

La diferencia de dependencias —tres frente a trescientas cuarenta y
nueve— es la medida concreta de lo que cuesta la generalidad, y la razón
por la que un zkVM contradice el principio de minimalismo de este
proyecto.

## Cuándo elegir cada uno

**Groth16** si el tamaño de prueba manda (192 bytes) y se puede celebrar
una ceremonia por circuito. Es además el más rápido de los tres basados
en curvas.

**PLONK-KZG** si se quiere un setup universal reutilizable con ceremonia
pública ya celebrada, pruebas pequeñas (1 KB) y verificación rápida, y se
puede asumir un generador lento.

**Halo2/IPA** si se quiere evitar toda ceremonia manteniendo pruebas
pequeñas (4 KB). Tras corregir la metodología, resulta el punto
intermedio más estrecho de lo que sugería la comparación anterior.

**STARK** si importan la transparencia total, la resistencia cuántica y
el rendimiento del generador, y se puede asumir el tamaño de prueba.

**Nova / plegado** si el volumen es alto y el cierre es periódico: el
coste por transacción es constante y el coste de cerrar se amortiza entre
todas. No sustituye a los otros — produce un estado plegado que hay que
comprimir con un SNARK, así que los necesita.

---

## Sobre el tamaño de prueba en un contexto bancario

El tamaño no bloquea la mensajería: los buses financieros (ISO 20022
sobre SWIFT, MQ, Kafka) mueven rutinariamente cargas de cientos de KB. La
fricción real es la **acumulación histórica**: a millones de
transacciones, almacenar **53,6-65,3 KB** por prueba —lo que miden los
circuitos de esta capa (§218), no los 36,7 KB del circuito de
comparación— en vez de 192 bytes cambia el tamaño del ledger, los
tiempos de sincronización y la validación en frío en dos órdenes de
magnitud.

---

## Limitaciones honestas de esta comparativa

- **Una sola ejecución por configuración, en una sola máquina.** Sin
  medias ni intervalos de confianza. Sirve para comparar órdenes de
  magnitud, no como benchmark riguroso.
- **Los circuitos no son idénticos.** Mismo diseño lógico, pero: el árbol
  STARK tiene 32 niveles frente a 20; los parámetros de Poseidon difieren
  entre backends (Hades de dusk, Poseidon de arkworks, Poseidon de
  halo2_gadgets); y la hoja de PLONK es un hash de aridad 4 mientras en
  los otros son dos hashes anidados.
- **Las pruebas no son intercambiables entre backends.** Cuerpos finitos
  y parámetros distintos: los árboles de Merkle y los espacios de
  nullifiers son incompatibles. Ver
  `crates/settlement-prover/src/lib.rs`.
- **`plonk-experiment` depende de una release candidate**
  (`dusk-poseidon 0.42.0-rc.0`), la única versión que compila en Rust
  estable junto a `dusk-plonk 0.22`. Y `dusk-plonk` es MPL-2.0, no
  MIT/Apache como el resto del proyecto.
- **Nada de esto ha sido auditado por terceros.**
