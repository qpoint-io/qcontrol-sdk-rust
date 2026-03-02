//! Pattern replacement types for network transforms
//!
//! Defines patterns for string replacement in transform pipelines.

/// A pattern for string replacement.
///
/// Used in `NetRwConfig` to define automatic replacements
/// that occur during send/recv operations.
#[derive(Debug, Clone)]
pub struct NetPattern {
    needle: Vec<u8>,
    replacement: Vec<u8>,
}

impl NetPattern {
    /// Create a new pattern from byte slices.
    pub fn new(needle: impl Into<Vec<u8>>, replacement: impl Into<Vec<u8>>) -> Self {
        Self {
            needle: needle.into(),
            replacement: replacement.into(),
        }
    }

    /// Create a new pattern from strings.
    pub fn from_str(needle: &str, replacement: &str) -> Self {
        Self::new(needle.as_bytes().to_vec(), replacement.as_bytes().to_vec())
    }

    /// Get the needle bytes.
    pub fn needle(&self) -> &[u8] {
        &self.needle
    }

    /// Get the replacement bytes.
    pub fn replacement(&self) -> &[u8] {
        &self.replacement
    }
}
