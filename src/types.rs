//! Core types: FilterResult, Error, FilterHandle

use std::fmt;

use crate::ffi;

/// Result of a filter callback, determining how the operation proceeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    /// Continue to the next filter in the chain
    Continue,
    /// Continue but apply any modifications made to the context
    Modify,
    /// Block the operation entirely (returns error to caller)
    Block,
}

/// Error codes from SDK operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An invalid argument was provided (e.g., name contains null bytes).
    InvalidArg,
    /// Memory allocation failed.
    NoMemory,
    /// The SDK is not initialized.
    NotInitialized,
    /// The specified plugin was not found.
    PluginNotFound,
    /// Plugin initialization failed.
    PluginInitFailed,
    /// Filter registration failed.
    RegisterFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidArg => write!(f, "invalid argument"),
            Error::NoMemory => write!(f, "memory allocation failed"),
            Error::NotInitialized => write!(f, "SDK not initialized"),
            Error::PluginNotFound => write!(f, "plugin not found"),
            Error::PluginInitFailed => write!(f, "plugin initialization failed"),
            Error::RegisterFailed => write!(f, "filter registration failed"),
        }
    }
}

impl std::error::Error for Error {}

/// Handle for a registered filter, used for unregistration.
#[derive(Debug, Clone, Copy)]
pub struct FilterHandle(pub(crate) ffi::qcontrol_filter_handle_t);
