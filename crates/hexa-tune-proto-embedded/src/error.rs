// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Domain-specific error types for hexaTune embedded.

use hexa_tune_proto::ProtoError;

/// hexaTune domain errors.
///
/// Wraps protocol errors and adds domain-specific variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexaError {
    /// A protocol-level error occurred.
    Proto(ProtoError),
    /// Command name not recognized in hexaTune command set.
    UnknownCommand,
    /// DDS subsystem is busy.
    DdsBusy,
    /// Command requires query mode but was sent as set (or vice versa).
    NotAQuery,
    /// A required parameter is missing.
    MissingParam,
    /// A parameter value is invalid.
    InvalidParam,
}

impl From<ProtoError> for HexaError {
    fn from(e: ProtoError) -> Self {
        Self::Proto(e)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for HexaError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Self::Proto(e) => defmt::write!(f, "Proto({})", e),
            Self::UnknownCommand => defmt::write!(f, "UnknownCommand"),
            Self::DdsBusy => defmt::write!(f, "DdsBusy"),
            Self::NotAQuery => defmt::write!(f, "NotAQuery"),
            Self::MissingParam => defmt::write!(f, "MissingParam"),
            Self::InvalidParam => defmt::write!(f, "InvalidParam"),
        }
    }
}
