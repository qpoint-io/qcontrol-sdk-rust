//! Server-Sent Events operation types.
//!
//! SSE callbacks sit above HTTP response delivery. The host owns parsing the
//! `text/event-stream` body into complete logical event records before the
//! plugin sees them.

mod event;

pub use event::{SseCloseEvent, SseEventRecord, SseOpenEvent};

// Re-export plugin state for convenience (used in callbacks)
pub use crate::state::{FileState, PluginState};

/// Callback type for the start of one SSE response stream.
pub type SseOpenFn = fn(&SseOpenEvent) -> crate::stream::StreamAction;

/// Callback type for one parsed SSE event record.
pub type SseEventFn = fn(PluginState, &mut SseEventRecord) -> crate::stream::StreamAction;

/// Callback type for the end of one SSE response stream.
pub type SseCloseFn = fn(PluginState, &SseCloseEvent);
