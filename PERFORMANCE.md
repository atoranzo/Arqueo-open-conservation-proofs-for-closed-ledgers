# Métricas de rendimiento — zk-ssl-real

> ⚠️ **DOCUMENTO SUPERADO.** Las cifras de este archivo se midieron en
> modo **debug** y son entre 11 y 14 veces peores que las reales. Las
> mediciones correctas, todas en release y con la misma metodología para
> los tres backends, están en [`FIVE_BACKENDS.md`](./FOUR_BACKENDS.md).
>
> Se conserva el documento en vez de borrarlo para dejar constancia del
> error metodológico y de su corrección.



Todos los números de este documento provienen de ejecuciones reales de
`cargo test`, no de estimaciones. Cada tiempo citado aquí tiene una
ejecución de test verificable detrás (ver `test_output.txt` en las
distintas rondas de esta conversación).

## ⚠️ Advertencia metodológica importante

Estos números se midieron con `cargo test --workspace` en **modo
paralelo** (el comportamiento por defecto), en un entorno WSL2 sobre
Windows con recursos compartidos. Cuando varios tests de Groth16 corren a
la vez, cada uno intenta usar todos los núcleos de CPU disponibles (vía
`rayon`), y se pisan entre sí — esto **infla artificialmente** los tiempos
individuales por encima de lo que costaría cada operación aislada.

No se ha medido todavía con `--test-threads=1` (ejecución en serie) para
obtener una línea base limpia sin contención. Los números de abajo son,
por tanto, **cotas superiores bajo contención de CPU**, no el mejor caso.
Marcado explícitamente como pendiente en la sección final.

## Resultados observados

| Operación | Circuito | Tiempo (contención baja) | Tiempo (contención alta) |
|---|---|---|---|
| Setup (`Groth16::Generator`) | `ComplianceCircuit` (sin estado) | ~4.0 – 5.0 s | ~6.6 – 10.8 s |
| Generar prueba (`Groth16::Prover`) | `ComplianceCircuit` (sin estado) | ~0.78 – 0.94 s | ~3.5 – 4.3 s |
| Setup (`Groth16::Generator`) | `ComplianceCircuitWithState` (con Poseidon, árbol de 20 niveles) | ~5.4 – 5.5 s | ~13.2 – 14.3 s |
| Generar prueba (`Groth16::Prover`) | `ComplianceCircuitWithState` (con Poseidon, árbol de 20 niveles) | ~4.9 – 5.1 s | ~10.7 – 11.7 s |

"Contención baja" = mediciones tomadas cuando ese test era el único o casi
el único ejecutándose en ese momento dentro de su binario. "Contención
alta" = mediciones tomadas cuando 5-6 tests pesados de Groth16 corrían a
la vez (el caso típico en `zk-core::circuit_with_state::tests`).

## Lo que NO está medido todavía

- **`verify()` / `verify_with_state()` de forma aislada.** Las trazas de
  `ark-std` (`print-trace`) envuelven `Groth16::Generator` (setup) y
  `Groth16::Prover` (generación de prueba), pero no `SNARK::verify`. Por
  diseño de Groth16, la verificación debería ser rápida (un número
  constante de operaciones de emparejamiento bilineal, independiente del
  tamaño del circuito) — probablemente milisegundos — pero esto es una
  expectativa teórica basada en el esquema, no un número medido con reloj
  en este proyecto todavía.
- **Línea base sin contención de CPU** (`--test-threads=1`). Pendiente,
  como se explica arriba.
- **Tamaño de la prueba en bytes.** Groth16 produce pruebas de tamaño
  constante (~200 bytes típicamente para BLS12-381), pero no se ha medido
  explícitamente serializando una prueba real de este proyecto.
- **Coste específico de Poseidon aislado** (cuánto de la diferencia entre
  `ComplianceCircuit` y `ComplianceCircuitWithState` es Poseidon puro,
  frente al resto de restricciones del árbol de Merkle — el rango, el
  nullifier, etc.).

## Lectura honesta de lo que sí sabemos

- **Poseidon cuesta, de forma clara y medible.** El salto de ~4-5s a
  ~5.4-5.5s en setup (contención baja) y de ~0.8-0.9s a ~4.9-5.1s en
  generación de prueba confirma que vincular el circuito a un árbol de
  Merkle de 20 niveles con Poseidon real tiene un coste no trivial, tal
  como advertía el propio BIS sobre las limitaciones de rendimiento de
  ZKPs en este tipo de aplicación.
- **El coste NO es prohibitivo** para un caso de uso de liquidación (no de
  alta frecuencia): unos segundos por transacción es aceptable para
  compliance de pagos de alto valor, que no requiere miles de operaciones
  por segundo como sí exigiría, por ejemplo, un sistema de pagos minoristas
  instantáneos.
- **La caché de parámetros de Poseidon fue la corrección de rendimiento
  más importante de todo el proyecto hasta ahora** (ver README): sin ella,
  cada prueba tardaba varios minutos en vez de varios segundos, por
  recalcular la matriz MDS decenas de veces.

## Próximo paso honesto

Antes de citar estos números en cualquier documento externo (preprint,
propuesta a un programa de financiación, etc.), habría que:
1. Medir con `--test-threads=1` para tener una cifra sin contención.
2. Medir `verify()` explícitamente con un cronómetro real, no asumir su
   coste por la teoría del esquema.
3. Repetir la medición varias veces y reportar media/desviación, no un
   solo dato por caso — la propia variabilidad observada aquí (5.4s vs
   14.3s para la misma operación) demuestra que una sola medición no es
   suficiente para caracterizar el rendimiento real del sistema.
