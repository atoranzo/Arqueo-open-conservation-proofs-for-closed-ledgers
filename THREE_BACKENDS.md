# Groth16 vs Halo2 vs STARK: comparativa con métricas reales

Todos los números de este documento se midieron ejecutando el **mismo
circuito de cumplimiento**, en la misma máquina (WSL2 sobre un portátil de
consumo) y **todos en modo release**. No hay cifras citadas de la
literatura ni estimaciones.

Reproducir:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
```

---

## Nota metodológica: por qué esta versión corrige a la anterior

Una versión previa de este documento comparaba cifras de Groth16 y Halo2
medidas en **debug** con cifras de STARK medidas en **release**. Al
homogeneizar la metodología, los dos primeros backends resultaron entre
**11 y 14 veces más rápidos** de lo que se les atribuía:

| | Cifra anterior (debug) | Cifra real (release) |
|---|---|---|
| Groth16, generación | ~5,2 s | **0,42 s** |
| Halo2, generación | ~53,5 s | **4,86 s** |
| Halo2, verificación | ~1,28 s | **0,09 s** |
| Halo2, setup | ~176 s | **16,3 s** |

La ventaja de STARK en generación sigue siendo real, pero es de **un orden
de magnitud**, no de dos. Se documenta el error en vez de corregirlo en
silencio.

---

## Tabla comparativa

| | Groth16 (BLS12-381) | Halo2 / IPA (Pallas) | STARK / FRI (Goldilocks) |
|---|---|---|---|
| **Paradigma** | R1CS | Plonkish | AIR |
| **Trusted setup** | Sí, por circuito | No | No |
| **Setup** | 438 ms | 16,3 s | **ninguno** |
| **Generación** | 422 ms | 4,86 s | **38 ms** |
| **Verificación** | 5 ms | 91 ms | **1 ms** |
| **Tamaño de prueba** | **192 bytes** | 4.096 bytes | 36,7 KB |
| **Resistencia cuántica** | No | No | **Sí** |
| **Profundidad del árbol** | 20 niveles | 20 niveles | 32 niveles |

Las cifras de STARK corresponden a la configuración de **127 bits
conjeturados** (blowup 16, extensión cuadrática), que es la comparable con
los ~128 bits de Groth16 y Halo2.

### Los tres ejes, en proporciones

- **Tamaño**: Groth16 gana por mucho. Una prueba STARK ocupa ~196 veces
  más que una de Groth16, y ~9 veces más que una de Halo2.
- **Velocidad de generación**: STARK gana. Es ~11 veces más rápido que
  Groth16 y ~128 veces más rápido que Halo2.
- **Coste de arranque**: STARK no tiene ninguno. Halo2 paga 16 segundos de
  parámetros; Groth16 paga menos tiempo pero exige una ceremonia (ver
  `crates/ceremony`).

---

## El matiz que cambia la comparación: conjeturada vs demostrable

Winterfell distingue dos estimaciones de seguridad, y su documentación
advierte que alcanzar el mismo nivel de forma demostrable exige de 2 a 3
veces más queries:

- **Conjeturada**: la que se cita habitualmente en el ecosistema STARK.
  Descansa en una conjetura sobre decodificación de Reed-Solomon que se
  considera razonable, pero que es una conjetura.
- **Demostrable**: solidez con demostración, sin esa conjetura.

Medido en nuestro circuito:

| Configuración | Tamaño | Generación | Conjeturada | Demostrable (udr/ldr) |
|---|---|---|---|---|
| blowup 8, sin extensión | 27,7 KB | 25 ms | 63 bits | 26 / 24 |
| blowup 8, ext. cuadrática | 32,7 KB | 28 ms | 95 bits | 26 / 47 |
| **blowup 16, ext. cuadrática** | **36,7 KB** | **38 ms** | **127 bits** | 29 / 63 |
| 120 queries, blowup 16, grinding 20, ext. cúbica | 125,6 KB | 45 ms | 128 bits | **128 / 128** |

Tres observaciones que no aparecen en los materiales promocionales
habituales:

1. **Sin extensión de campo, el techo son 63 bits.** El campo Goldilocks
   mide 64 bits y la solidez no puede superarlo por muchas queries que se
   añadan. La configuración "rápida y compacta" que uno elegiría por
   defecto no es comparable con Groth16 o Halo2.
2. **La brecha entre conjeturada y demostrable es enorme** en las
   configuraciones cómodas: 127 bits conjeturados conviven con 29-63
   demostrables.
3. **Pero cerrar esa brecha es asequible**: 128 bits demostrables cuestan
   125,6 KB y 45 ms — todavía ~9 veces más rápido en generación que
   Groth16, sin ceremonia y sin conjeturas.

---

## El coste de la garantía contable

El circuito de partida doble (`circuit_double_entry`) demuestra la
conservación del dinero, no solo la solvencia del emisor. Medido en
Groth16:

| | Solvencia | Partida doble | Factor |
|---|---|---|---|
| Restricciones | 9.934 | 27.562 | 2,8× |
| Setup | 438 ms | 1.123 ms | 2,6× |
| Generación | 422 ms | 1.170 ms | 2,8× |
| Verificación | 5 ms | 5 ms | **1×** |
| Tamaño | 192 bytes | 192 bytes | **1×** |

El coste crece de forma casi proporcional a las restricciones en el lado
del generador, y **es nulo en el lado del verificador**: una prueba
Groth16 mide siempre lo mismo y se verifica en el mismo tiempo, sea cual
sea el circuito. En una red de liquidación con muchos más verificadores
que generadores, esa asimetría importa.

---

## Cuándo elegir cada uno

**Groth16** si el tamaño de la prueba manda por encima de todo (192 bytes,
dos órdenes de magnitud menos que cualquier alternativa) y se puede
celebrar una ceremonia MPC real. El mecanismo de ceremonia existe y está
verificado en `crates/ceremony`; celebrarla con participantes
independientes sigue pendiente.

**Halo2** si se quiere evitar la ceremonia manteniendo pruebas pequeñas
(4 KB). Es el punto intermedio, aunque tras la corrección metodológica
resulta ser el más lento de los tres tanto en setup como en generación.

**STARK** si importan la transparencia total, la resistencia cuántica y el
rendimiento del generador, y se puede asumir el tamaño de prueba. Perfil
opuesto al de Groth16: pruebas grandes, todo lo demás rápido, cero setup.

---

## Sobre el tamaño de prueba en un contexto bancario

El tamaño no bloquea la mensajería: los buses financieros (ISO 20022 sobre
SWIFT, MQ, Kafka) mueven rutinariamente cargas de cientos de KB. La
fricción real es la **acumulación histórica**: a millones de
transacciones, almacenar 36,7 KB por prueba en vez de 192 bytes cambia el
tamaño del ledger persistente, los tiempos de sincronización entre nodos y
la validación en frío en dos órdenes de magnitud.

Ese es el eje real de la decisión, y depende del volumen y de la política
de retención de cada despliegue.

---

## Limitaciones honestas de esta comparativa

- **Una sola ejecución por configuración, en una sola máquina.** No hay
  medias de repeticiones ni intervalos de confianza. Estos números sirven
  para comparar órdenes de magnitud, no como benchmark riguroso.
- **Los circuitos no son idénticos**: mismo diseño lógico, pero el árbol
  STARK tiene 32 niveles frente a 20 y usa Rescue en vez de Poseidon. El
  backend STARK hace, por tanto, *más* trabajo de hash, no menos — su
  ventaja en generación está medida en desventaja.
- **El circuito de partida doble solo existe en Groth16.** Las cifras de
  Halo2 y STARK corresponden al circuito de solvencia.
- **Las pruebas no son intercambiables entre backends**: cada uno opera
  sobre un cuerpo finito distinto, así que los árboles de Merkle y los
  espacios de nullifiers son incompatibles. Ver
  `crates/settlement-prover/src/lib.rs`.
- **Nada de esto ha sido auditado por terceros.**
