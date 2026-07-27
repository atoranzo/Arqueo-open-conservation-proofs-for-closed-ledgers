# Groth16 vs. Halo2 — comparativa final, con datos reales

Este documento compara los dos motores criptográficos completos y
verificados de este proyecto: `zk-core` (Arkworks/Groth16) y
`halo2-experiment` (Halo2/IPA). Ambos implementan la misma lógica
(cumplimiento con range checks + árbol de Merkle de 20 niveles +
nullifier + puente ISO 20022), verificados de extremo a extremo con
pruebas criptográficas reales, no solo satisfacibilidad.

## Tabla comparativa

| Aspecto | Groth16 (`zk-core`) | Halo2 (`halo2-experiment`) |
|---|---|---|
| Trusted setup | **Sí, por circuito.** Un solo participante en este proyecto — limitación de seguridad real y sin resolver (ver README de `zk-core`). | **No.** Setup determinista vía IPA; cualquiera puede regenerarlo y comprobar que coincide. |
| Tiempo de setup | Segundos (varía con contención de CPU, ver `PERFORMANCE.md`). | **176 s** (`k=15`) — mucho más caro, pero se paga UNA VEZ para cualquier circuito de ese tamaño. |
| `keygen` | Incluido en el setup. | `keygen_vk`: 20 s. `keygen_pk`: 15 s. |
| Generar una prueba | Unos segundos (circuito con estado, con contención de CPU). | **53.5 s** — notablemente más lento. |
| Verificar una prueba | Rápido (no medido por separado, pero teóricamente sub-segundo). | **1.28 s** — rápido en términos absolutos, algo más lento que Groth16. |
| Tamaño de la prueba | ~200 bytes (típico de Groth16). | **4.096 bytes** — ~20x más pesada. |
| Madurez del ecosistema de circuitos | Arkworks, muy usado en investigación académica. `ark-groth16` se declara oficialmente "prototipo académico, no listo para producción". | Halo2 (zcash/halo2) está en producción real (Zcash Orchard) y auditado. |
| Herramientas de ceremonia MPC | Investigadas dos veces (`ark-marlin`, `celo-org/snark-setup`); ambas resultaron abandonadas/incompatibles con la generación actual del ecosistema — callejón sin salida confirmado. | No aplica — no hace falta ceremonia. |
| Complejidad del código de circuito | Media (R1CS + gadgets de Arkworks). | Alta (arquitectura Plonkish: puertas personalizadas, columnas fijas/advice, selectors, regiones) — más control, más superficie de error. |
| Persistencia de nullifiers | ✅ `sled`, verificada. | ✅ `sled`, verificada (mismo diseño, adaptado a `ff::PrimeField`). |

## Lectura honesta

**Ninguno de los dos motores "gana" en términos absolutos** — es el
trade-off clásico y ya conocido en la literatura de SNARKs, ahora
confirmado con datos propios:

- **Groth16 es la opción correcta si**: el rendimiento (pruebas rápidas y
  pequeñas) importa más que eliminar el trusted setup, y se está
  dispuesto a asumir el riesgo de seguridad de una ceremonia de un solo
  participante hasta que se resuelva correctamente (ver el hueco
  documentado en el README de `zk-core`).
- **Halo2 es la opción correcta si**: eliminar el trusted setup es un
  requisito no negociable (por ejemplo, para credibilidad ante un
  regulador que específicamente objete a esa limitación), y se puede
  tolerar que las pruebas sean ~10 veces más lentas de generar y ~20
  veces más pesadas.

## Qué NO cambia esta comparación

Ninguno de los dos resuelve, por sí solo, los otros huecos ya
documentados en ambos proyectos: coordinación distribuida del registro de
nullifiers entre varios validadores, parser ISO 20022 completo (ambos
usan el mismo subconjunto simplificado), ni auditoría externa. Esta
comparación es exclusivamente sobre el motor de pruebas criptográficas,
no sobre la preparación del proyecto para producción en general.

## Gobernanza y ciclo de vida institucional (más allá del rendimiento puro)

Esta sección responde a una pregunta distinta a la de arriba: no "¿cuál es
más rápido hoy?", sino "¿cuál es más ágil de mantener en una institución
real, a lo largo de años, cuando las reglas de cumplimiento cambian?".

### Por qué el tiempo de verificación NO es "milisegundos en ambos"

Es una simplificación habitual en la literatura general de SNARKs, pero
**nuestros propios datos la contradicen para Halo2/IPA**: medimos
`verify_proof` en **1.28 segundos**, no milisegundos — más de mil veces
más lento que la expectativa típica.

La razón es técnica, no un fallo de nuestra implementación: la
verificación en **IPA** (el esquema que usa `zcash/halo2`, elegido
precisamente para no necesitar trusted setup) escala de forma
**aproximadamente lineal** con el tamaño del circuito. La verificación en
**KZG** (el esquema que usa Groth16, y que también podría usar una
variante de Halo2/PLONK) es casi constante. Es el mismo trade-off de
"sin ceremonia ↔ más lento" que ya vimos en la generación de pruebas,
pero manifestándose también en la verificación — un matiz que se pierde
si se asume "ambos verifican en milisegundos" sin más. Si en el futuro se
optara por el fork de PSE con KZG en vez de IPA, la verificación
probablemente sería más rápida, pero eso reintroduciría la necesidad de
un setup universal — no existe una combinación que dé lo mejor de ambos
mundos gratis.

### "Los cambios de circuito son instantáneos" — cierto, pero no del todo

Sin ceremonia MPC de por medio, un cambio de circuito en Halo2 es mucho
más rápido que en Groth16 — pero **no instantáneo**. Nuestros propios
datos: `keygen_vk` (20 s) + `keygen_pk` (15 s) ≈ **35 segundos** tras
cualquier cambio de circuito. Sigue siendo una diferencia enorme frente a
las semanas o meses de coordinar una ceremonia — "minutos, no meses" es
la comparación honesta, no "instantáneo".

### Lo que eliminar la ceremonia NO elimina: la fricción de gobernanza

Este es el punto más importante que añadir al análisis de rendimiento
puro. Eliminar la ceremonia criptográfica **no elimina la necesidad de
revisión y auditoría** de un circuito nuevo antes de confiar en él en
producción. Si un banco cambia una regla de Basilea III/IV, ese nuevo
circuito necesita pasar por control de cambios, revisión de seguridad y
probablemente aprobación regulatoria — tenga o no ceremonia MPC de por
medio. Para una institución real, es razonable esperar que **ese proceso
de gobernanza sea el cuello de botella dominante**, no la criptografía en
sí. La "agilidad temporal infinita" de Halo2 es real a nivel de
infraestructura técnica, pero no resuelve por sí sola la fricción
organizativa que rodea cualquier cambio de lógica de cumplimiento en un
entorno regulado.

### Qué implementación concreta de Groth16 se está comparando

"Groth16 tiene herramientas maduras" es cierto para el ecosistema
**circom/snarkjs** (el que usan Tornado Cash, Zcash Sprout/Sapling, con
ceremonias públicas reales como Perpetual Powers of Tau). Es más
discutible para **Arkworks**, que es lo que usa este proyecto: el propio
repositorio de `ark-groth16` se declara *"prototipo académico, no listo
para producción"*, y las dos herramientas de ceremonia MPC que
investigamos para este ecosistema concreto (`ark-marlin`,
`celo-org/snark-setup`) resultaron abandonadas e incompatibles con la
generación actual de Arkworks (ver investigación documentada más arriba
en esta conversación). La madurez de "Groth16" depende mucho de qué
implementación concreta se evalúe — no es un bloque monolítico.

## Recomendación práctica

Si el objetivo es seguir con **un solo** motor de aquí en adelante (para
no mantener dos bases de código en paralelo), la elección razonable
depende del público:

- Para una **demostración técnica o preprint académico** dirigido a
  comunidades de investigación: Halo2, porque elimina la objeción más
  fácil de plantear ("¿y el trusted setup?").
- Para un **prototipo de compliance-as-a-service** donde la latencia de
  respuesta importa (recordando la conversación sobre monetización):
  Groth16, porque generar pruebas 10 veces más rápido es una ventaja de
  producto real, y el trusted setup puede resolverse más adelante con una
  ceremonia MPC seria cuando haya recursos para ello.

No hay una respuesta correcta única — es una decisión de producto, no una
limitación técnica que quede por resolver.

## Metodología de esta comparación (para que sea verificable, no solo afirmada)

Todos los números de Halo2 provienen de una única ejecución real medida
con `Instant::now()` en `compliance_real_proof.rs` (`k=15`, sin
contención de CPU relevante al ejecutarse en solitario). Los números de
Groth16 provienen de las ejecuciones de `cargo test` documentadas en
`PERFORMANCE.md`, con la advertencia allí señalada sobre contención de
CPU en paralelo. **Antes de citar esta comparación en cualquier documento
externo**, sería necesario repetir ambas mediciones bajo las mismas
condiciones controladas (mismo hardware, sin contención, varias
repeticiones con media/desviación) — esta es una comparación honesta de
orden de magnitud, no un benchmark riguroso.
