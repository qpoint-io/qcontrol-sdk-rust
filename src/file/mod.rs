//! File operation types
//!
//! This module provides types for file operation filtering:
//! - Events for open/read/write/close operations
//! - Session configuration for per-file transforms

mod action;
mod event;
mod pattern;
mod session;

// Re-export all public types
pub use action::{FileAction, FileOpAction, FileOpenResult};
pub use event::{
    FileCloseEvent, FileOpEvent, FileOpKind, FileOpenEvent, FilePolicyClass, FileReadEvent,
    FileWriteEvent,
};
pub use pattern::FilePattern;
pub use session::{
    FileContext, FileRwConfig, FileSession, FileSessionBuilder, FileTransformFn, SessionState,
};

use crate::state::PluginState;

// Re-export Buffer from parent module for convenience
pub use crate::buffer::Buffer;

/// Callback type for file open events.
///
/// Receives the open event and returns an action determining how to handle the file.
pub type FileOpenFn = fn(&FileOpenEvent) -> FileOpenResult;

/// Callback type for file read events.
///
/// Receives the session state and read event, returns an action.
pub type FileReadFn = fn(PluginState, &FileReadEvent) -> FileAction;

/// Callback type for file write events.
///
/// Receives the session state and write event, returns an action.
pub type FileWriteFn = fn(PluginState, &FileWriteEvent) -> FileAction;

/// Callback type for file close events.
///
/// Receives the session state and close event. Called for cleanup.
pub type FileCloseFn = fn(PluginState, &FileCloseEvent);

/// Callback type for generic path-oriented filesystem operations.
pub type FileOpFn = fn(&FileOpEvent) -> FileOpAction;
