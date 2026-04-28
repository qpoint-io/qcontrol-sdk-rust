//! Exec operation types
//!
//! This module provides types for exec operation filtering:
//! - Events for exec/stdin/stdout/stderr/exit operations
//! - Session configuration for per-process transforms
//! - State management for tracking data across operations

mod action;
mod context;
mod event;
mod pattern;
mod session;

// Re-export all public types
pub use action::{ExecAction, ExecResult};
pub use context::ExecContext;
pub use event::{ExecEvent, ExitEvent, StderrEvent, StdinEvent, StdoutEvent};
pub use pattern::ExecPattern;
pub use session::{ExecRwConfig, ExecSession, ExecSessionBuilder, ExecTransformFn, SessionState};

// Re-export plugin state for convenience (used in callbacks)
pub use crate::state::{FileState, PluginState};

// Re-export Buffer from parent module for convenience
pub use crate::buffer::Buffer;

/// Callback type for exec events.
///
/// Receives the exec event and returns an action determining how to handle the process.
pub type ExecFn = fn(&ExecEvent) -> ExecResult;

/// Callback type for stdin events.
///
/// Receives the session state and stdin event, returns an action.
pub type ExecStdinFn = fn(PluginState, &StdinEvent) -> ExecAction;

/// Callback type for stdout events.
///
/// Receives the session state and stdout event, returns an action.
pub type ExecStdoutFn = fn(PluginState, &StdoutEvent) -> ExecAction;

/// Callback type for stderr events.
///
/// Receives the session state and stderr event, returns an action.
pub type ExecStderrFn = fn(PluginState, &StderrEvent) -> ExecAction;

/// Callback type for exit events.
///
/// Receives the session state and exit event. Called for cleanup.
pub type ExecExitFn = fn(PluginState, &ExitEvent);
