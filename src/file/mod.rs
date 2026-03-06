//! File operation types
//!
//! This module provides types for file operation filtering:
//! - Events for open/read/write/close operations
//! - Session configuration for per-file transforms
//! - State management for tracking data across operations

mod action;
mod event;
mod pattern;
mod session;
mod state;

// Re-export all public types
pub use action::{FileAction, FileOpenResult};
pub use event::{FileCloseEvent, FileOpenEvent, FileReadEvent, FileWriteEvent};
pub use pattern::FilePattern;
pub use session::{DeclaredTransforms, FileContext, FileRwConfig, FileSession, FileSessionBuilder, FileTransformFn, SessionState};
pub use state::FileState;

// Re-export Buffer from parent module for convenience
pub use crate::buffer::Buffer;

/// Callback type for file open events.
///
/// Receives the open event and returns an action determining how to handle the file.
pub type FileOpenFn = fn(&FileOpenEvent) -> FileOpenResult;

/// Callback type for file read events.
///
/// Receives the session state and read event, returns an action.
pub type FileReadFn = fn(FileState, &FileReadEvent) -> FileAction;

/// Callback type for file write events.
///
/// Receives the session state and write event, returns an action.
pub type FileWriteFn = fn(FileState, &FileWriteEvent) -> FileAction;

/// Callback type for file close events.
///
/// Receives the session state and close event. Called for cleanup.
pub type FileCloseFn = fn(FileState, &FileCloseEvent);
