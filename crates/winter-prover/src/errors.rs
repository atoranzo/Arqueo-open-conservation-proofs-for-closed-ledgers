// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Contains common error types for prover and verifier.

use core::fmt;

// PROVER ERROR
// ================================================================================================
/// Represents an error returned by the prover during an execution of the protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverError {
    /// This error occurs when a transition constraint evaluated over a specific execution trace
    /// does not evaluate to zero at any of the steps.
    UnsatisfiedTransitionConstraintError(usize),
    /// This error occurs when polynomials built from the columns of a constraint evaluation
    /// table do not all have the same degree.
    MismatchedConstraintPolynomialDegree(usize, usize),
    /// This error occurs when the base field specified by the AIR does not support field extension
    /// of degree specified by proof options.
    UnsupportedFieldExtension(usize),
    /// ARQUEO (RFC-0009 D-AH): el ancho de la traza no es el del AIR (esperado, real).
    AnchoDiscordante { esperado: usize, real: usize },
    /// ARQUEO (RFC-0009 D-AH): una asercion del AIR no se cumple en la traza (columna, paso).
    /// El probador oculto comprueba la traza real antes de probarla, tambien en release: un
    /// testigo malo es un error del cliente, no un panico.
    AsercionNoSatisfecha { columna: usize, paso: usize },
    /// ARQUEO (RFC-0009 D-AH): una restriccion de transicion del tramo auxiliar no se anula
    /// (indice de la restriccion, paso).
    RestriccionAuxNoSatisfecha { indice: usize, paso: usize },
    /// ARQUEO (RFC-0009 D-AG, D-AH): m no es menor que 2T (m, filas de la traza oculta): la
    /// traza es corta para ese m.
    OcultacionNoCabe { m: usize, filas: usize },
    /// ARQUEO (RFC-0009 D-AH): la segunda cota de las exenciones del envoltorio no cabe ni con
    /// el blowup del LDE: no se prueba.
    EnvoltorioNoCabe,
}

impl fmt::Display for ProverError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsatisfiedTransitionConstraintError(step) => {
                write!(f, "a transition constraint was not satisfied at step {step}")
            }
            Self::MismatchedConstraintPolynomialDegree(expected, actual) => {
                write!(f, "the constraint polynomial's components do not all have the same degree; expected {expected}, but was {actual}")
            }
            Self::UnsupportedFieldExtension(degree) => {
                write!(f, "field extension of degree {degree} is not supported for the specified base field")
            }
            Self::AnchoDiscordante { esperado, real } => {
                write!(f, "ancho de la traza {real}; el AIR espera {esperado}")
            }
            Self::AsercionNoSatisfecha { columna, paso } => {
                write!(f, "la traza no cumple la asercion en la columna {columna}, paso {paso}")
            }
            Self::RestriccionAuxNoSatisfecha { indice, paso } => {
                write!(f, "la restriccion auxiliar {indice} no se anula en el paso {paso}")
            }
            Self::OcultacionNoCabe { m, filas } => {
                write!(f, "traza oculta: m = {m} no cabe en la traza de {filas} filas")
            }
            Self::EnvoltorioNoCabe => {
                write!(f, "traza oculta: la segunda cota de las exenciones no cabe ni con el blowup")
            }
        }
    }
}

impl core::error::Error for ProverError {}
