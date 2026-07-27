# Atribución del código vendorizado

Este crate **no es código original de este proyecto**. Su contenido en
`src/` procede del crate [`penumbra-sdk-proof-setup`][crate] versión
**2.1.1**, obra de **Penumbra Labs** ([penumbra-zone/penumbra][repo]),
publicado bajo licencia dual **MIT / Apache-2.0** — la misma que usa este
repositorio, lo que hace la integración legalmente limpia.

Se ha vendorizado únicamente el módulo `single/`, que implementa la
ceremonia MPC de dos fases (BGM17) para Groth16 sobre arkworks. Se han
descartado `all.rs` y toda la integración con el SDK de Penumbra
(`penumbra-sdk-dex`, `-governance`, `-shielded-pool`, `-stake`), que son
específicos de sus circuitos.

## Ficheros incluidos

| Fichero | Contenido |
|---|---|
| `phase1.rs` | Powers of Tau (fase universal, reutilizable entre circuitos) |
| `phase2.rs` | Especialización al circuito concreto |
| `dlog.rs` | Pruebas de conocimiento del logaritmo discreto (validan cada contribución) |
| `log.rs` | Transcript de contribuciones y sus hashes |
| `group.rs` | Alias de tipos del grupo — **el único punto donde vive la curva** |
| `parallel_utils.rs` | Utilidades de paralelización |
| `lib.rs` | Antes `mod.rs`: API pública y funciones `transition`/`combine` |

## Modificaciones realizadas

Se ha mantenido la estructura de módulos original (`src/single/`) para
que el diff contra upstream sea mínimo y auditable. Los cambios son
**exclusivamente de imports y del alias de curva**; ninguna línea de
lógica criptográfica ha sido tocada.

### 1. Cambio de curva (2 líneas + renombrados mecánicos)

El crate original está fijado a **BLS12-377** (vía `decaf377::Bls12_377`).
Aquí se ha reapuntado a **BLS12-381**, la curva de `zk-core`:

| Fichero | Cambio |
|---|---|
| `single/group.rs` | `use decaf377::Bls12_377;` → `use ark_bls12_381::Bls12_381 as Curve;` |
| `single/mod.rs` | idem |
| todos | renombrado mecánico `Bls12_377` → `Curve` |

Se renombró el identificador a `Curve` en vez de dejar el nombre
`Bls12_377` apuntando a otra curva — eso habría sido una trampa para
cualquiera que leyese el código después.

### 2. Imports de traits que aportaba `decaf377` (4 líneas)

Al sustituir `decaf377` por `ark_bls12_381` puro se perdieron varios
traits que estaban en ámbito de forma indirecta. Hubo que declararlos:

| Fichero | Añadido | Motivo |
|---|---|---|
| `single/group.rs` | `use ark_ff::PrimeField;` | `F::from_le_bytes_mod_order` |
| `single/dlog.rs` | `use ark_ff::UniformRand;` | `F::rand` |
| `single/phase1.rs` | `UniformRand` al import existente | `F::rand` |
| `single/phase2.rs` | `Field`, `UniformRand` al import existente | `.inverse()`, `F::rand` |

**Por qué estos cambios son seguros**: importar un trait no puede alterar
la semántica del código. No existe ningún método inherente compitiendo
—por eso fallaba la compilación—, así que la única implementación
candidata es la del trait. O compila con la correcta, o no compila; no
hay fallo silencioso posible.

### 3. Ficheros nuevos, propios de este proyecto

- `src/lib.rs`: solo cabecera de atribución y declaración de módulos.
- `Cargo.toml`: dependencias equivalentes a las del original, sin los
  crates del SDK de Penumbra.
- Este `ATTRIBUTION.md`.

## Verificación: qué evidencia respalda esta adaptación

Los **34 tests del código original pasan sobre BLS12-381**, y entre ellos
están los tests negativos que comprueban las propiedades de solidez de la
ceremonia, no solo el camino feliz:

- `test_changing_alpha_makes_crs_invalid`
- `test_changing_beta_makes_crs_invalid`
- `test_changing_delta_makes_crs_invalid`
- `test_setting_zero_elements_makes_crs_invalid`
- `test_not_inverting_delta_makes_crs_invalid`
- `test_contribution_is_not_linked_to_itself`
- `test_bad_statement_makes_proof_fail`
- `test_can_generate_keys_through_ceremony` (ceremonia completa → prueba
  Groth16 verificada)

Reproducir: `cargo test -p ceremony --release`

## ⚠️ Advertencias que deben leerse antes de usar esto

1. **La adaptación no está auditada.** Penumbra ejerció su versión sobre
   BLS12-377 en producción. Esta versión sobre BLS12-381 es una
   adaptación de este proyecto, y los tests del punto anterior son la
   única verificación existente. Que pasen es evidencia fuerte, no una
   auditoría.

2. **BLS12-377 y BLS12-381 no son equivalentes.** La primera tiene el
   campo base altamente 2-ádico y la segunda no; solo el campo escalar lo
   es en ambas. Las FFT de la ceremonia operan sobre el campo escalar, y
   los tests confirman empíricamente que el cambio no rompe nada — pero
   conviene saber que la diferencia existe.

3. **`combine` construye la clave de verificación con γ = 1**
   (`gamma_g2: G2::generator()`). Es una decisión de diseño del código
   original, válida pero no la formulación de libro de Groth16. Las
   claves resultantes son coherentes consigo mismas; conviene saberlo
   antes de intentar interoperar con otras implementaciones.

4. **Una ceremonia MPC solo es segura si los participantes son reales e
   independientes.** Ejecutar las contribuciones en la misma máquina y en
   el mismo proceso —como hacen los tests— demuestra que el *mecanismo*
   funciona, **no** que exista garantía de seguridad: si toda la
   aleatoriedad la genera el mismo proceso, no hay ninguna parte honesta
   independiente. Una ceremonia real requiere participantes distintos, en
   máquinas distintas, publicando el transcript para que cualquiera pueda
   verificarlo.

[crate]: https://crates.io/crates/penumbra-sdk-proof-setup
[repo]: https://github.com/penumbra-zone/penumbra
