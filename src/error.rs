//! Error types for qcontrol SDK

use std::fmt;

/// Error codes from SDK operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An invalid argument was provided (e.g., name contains null bytes).
    InvalidArg,
    /// An invalid name was provided.
    InvalidName,
    /// Memory allocation failed.
    NoMemory,
    /// Plugin initialization failed.
    InitFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidArg => write!(f, "invalid argument"),
            Error::InvalidName => write!(f, "invalid name"),
            Error::NoMemory => write!(f, "memory allocation failed"),
            Error::InitFailed => write!(f, "plugin initialization failed"),
        }
    }
}

impl std::error::Error for Error {}
