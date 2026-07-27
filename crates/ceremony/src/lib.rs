//! Ceremonia MPC de dos fases (BGM17) para Groth16 sobre arkworks.
//!
//! CODIGO VENDORIZADO - NO ES ORIGINAL DE ESTE PROYECTO.
//!
//! Procede de `penumbra-sdk-proof-setup` v2.1.1, obra de Penumbra Labs,
//! bajo licencia dual MIT/Apache-2.0.
//!
//! El contenido de `single/` es IDENTICO al original salvo por dos
//! lineas: el import de la curva, reapuntado de BLS12-377 a BLS12-381.
//! Se ha mantenido deliberadamente la estructura de modulos original
//! para que el diff contra upstream sea minimo y auditable.
//!
//! Lee `ATTRIBUTION.md` antes de usar esto: contiene advertencias reales
//! sobre el alcance de la adaptacion y sobre que demuestra (y que NO
//! demuestra) ejecutar una ceremonia en una sola maquina.

mod parallel_utils;
mod single;

pub use single::*;
