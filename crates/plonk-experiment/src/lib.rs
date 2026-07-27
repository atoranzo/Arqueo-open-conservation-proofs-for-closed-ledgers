//! Cuarto backend: **PLONK con compromiso KZG** sobre BLS12-381, vía
//! `dusk-plonk`.
//!
//! ## Por qué este backend, tras descartar cinco vías
//!
//! El recorrido hasta llegar aquí está documentado en `README.md`. En
//! resumen: `plonk-core` (ZK-Garage) nunca se publicó en crates.io; el
//! gadget de Poseidon de PSE no tiene especificación para BN254;
//! `halo2-lib` de Axiom depende de repositorios git sin `rev` fijado; y
//! la combinación `dusk-plonk 0.21` + `dusk-poseidon 0.41` arrastra un
//! `msgpacker` que exige Rust **nightly**.
//!
//! La combinación que sí funciona en estable es
//! `dusk-plonk 0.22.1` + `dusk-poseidon 0.42.0-rc.0`.
//!
//! ## La ventaja diferencial: setup universal ya celebrado
//!
//! PLONK usa un trusted setup **universal y actualizable**: un solo
//! Powers of Tau sirve para TODOS los circuitos. Groth16, en cambio,
//! exige repetir la fase 2 para cada circuito distinto — y este proyecto
//! ya tiene dos (solvencia y partida doble).
//!
//! Y existe una ceremonia pública real sobre BLS12-381 coordinada por
//! Dusk, con herramienta de conversión al formato `PublicParameters`.
//! Frente a Groth16, donde el mecanismo de ceremonia está implementado
//! pero **no celebrado**, aquí habría una ceremonia ya hecha.
//!
//! **Matiz honesto**: sería la ceremonia de Dusk, no la de Ethereum ni
//! Perpetual Powers of Tau (esas son sobre BN254). La confianza descansa
//! en sus participantes. Sigue siendo incomparablemente mejor que un
//! `setup()` de una sola parte, pero conviene decirlo con precisión.
//!
//! ## Lo que este backend trae ya resuelto
//!
//! A diferencia de los otros tres, `dusk-plonk` incluye componentes que
//! allí hubo que escribir a mano:
//! - `component_range`: range checks nativos.
//! - `component_select` y `component_boolean`: selección condicional,
//!   que es lo que necesita la subida del árbol de Merkle.
//!
//! Eso reduce el alcance del port de forma apreciable.
//!
//! ## Estado
//!
//! - `minimal`: circuito mínimo (a + b = público). ✅ verificado 3/3.
//!   Tamaño de prueba medido: **1.008 bytes**, constante (el tipo es
//!   `[u8; N]`, no `Vec`). Queda entre Groth16 (192) y Halo2 (4.096).
//! - `poseidon_hash`: hash Poseidon nativo y en-circuito, con separación
//!   de dominio. ✅ verificado 5/5. **994 puertas por hash de aridad 2** —
//!   bastante más que en Groth16, lo que anticipa un circuito completo
//!   mayor.
//! - `merkle`: verificación de camino de 20 niveles en-circuito. ✅
//!   verificado 5/5. **19.946 puertas**.
//! - `compliance_circuit`: circuito de cumplimiento completo (solvencia,
//!   pertenencia y nullifier). ✅ verificado 6/6. **21.983 puertas**, y
//!   prueba de **1.008 bytes** — el mismo tamaño que el circuito mínimo
//!   de 3 puertas, lo que confirma que es CONSTANTE.
//! - `circuit_double_entry`: partida doble. ✅ verificado 7/7.
//!   **84.801 puertas**.
//! - `iso_bridge`: puente ISO 20022 pacs.008 + implementación del trait
//!   `SettlementProver`. ✅ verificado.
//! - `persistent_nullifier_registry`: registro sled contra doble gasto.
//!   ✅ verificado.
//! - `performance`: métricas con la misma metodología que los otros tres
//!   backends. ✅ verificado.
//!
//! **Suite completa: 36/36.**
//!
//! ## Tamaños medidos de circuito
//!
//! | Circuito | Puertas |
//! |---|---|
//! | Mínimo (a + b) | 3 |
//! | Hash Poseidon (aridad 2) | 994 |
//! | Árbol de Merkle (20 niveles) | 19.946 |
//! | Cumplimiento completo | 21.983 |
//! | Partida doble | 84.801 |
//!
//! La prueba mide **1.008 bytes en todos los casos**: el tamaño es
//! constante e independiente del circuito, igual que en Groth16 (192
//! bytes) y a diferencia de STARK.
//!
//! ## Dos observaciones de coste, medidas
//!
//! **El hash es ~3,3 veces más caro que en Groth16** (997 puertas frente
//! a ~300 restricciones por hash de aridad 2). Es lo que domina el tamaño
//! del circuito.
//!
//! **El range check es mucho más barato**: `component_range` nativo
//! cuesta ~16 puertas para 64 bits, frente a las 64-256 restricciones que
//! costaba construirlo a mano en los otros tres backends.
//!
//! ⚠️ **Ejecutar los tests SIEMPRE en `--release`.** Los circuitos de este
//! backend son grandes y en debug tardan varios minutos.

pub mod circuit_double_entry;
pub mod compliance_circuit;
pub mod iso_bridge;
pub mod merkle;
pub mod minimal;
pub mod performance;
pub mod persistent_nullifier_registry;
pub mod poseidon_hash;

#[cfg(test)]
pub mod test_support;
