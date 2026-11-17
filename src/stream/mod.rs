//! Shared derived-stream types.
//!
//! This module holds the correlation context and action surface shared by
//! higher-level protocols that are derived from one network connection and one
//! owning HTTP exchange.

mod action;
mod context;

pub use action::StreamAction;
pub use context::{StreamContext, StreamDirection, StreamProtocol};

/// State wrapper for derived stream callback state.
///
/// Mirrors the HTTP exchange state wrapper, but applies to stream-oriented
/// callback families such as SSE and WebSocket.
#[doc(hidden)]
pub struct StreamState {
    /// User-provided state (may be None).
    pub user_state: Option<Box<dyn std::any::Any + Send>>,
}

impl StreamState {
    /// Get a PluginState referencing the user's state.
    pub fn as_plugin_state(&self) -> crate::PluginState<'_> {
        match &self.user_state {
            Some(boxed) => crate::PluginState::from_ref(boxed.as_ref()),
            None => crate::PluginState::empty(),
        }
    }
}
