# Cuatro backends criptográficos: comparativa con métricas reales

Todos los números se midieron ejecutando el **mismo circuito de
cumplimiento**, en la misma máquina (WSL2 sobre un portátil de consumo) y
**todos en modo release**. No hay cifras citadas de la literatura.

Reproducir:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
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

### 5. AIR carece de restricciones de copia, y eso obliga a rediseñar

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

---

## Sobre el tamaño de prueba en un contexto bancario

El tamaño no bloquea la mensajería: los buses financieros (ISO 20022
sobre SWIFT, MQ, Kafka) mueven rutinariamente cargas de cientos de KB. La
fricción real es la **acumulación histórica**: a millones de
transacciones, almacenar 36,7 KB por prueba en vez de 192 bytes cambia el
tamaño del ledger, los tiempos de sincronización y la validación en frío
en dos órdenes de magnitud.

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
