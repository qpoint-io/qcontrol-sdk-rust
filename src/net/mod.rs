//! Network operation types
//!
//! This module provides types for network operation filtering:
//! - Events for connect/accept/tls/domain/protocol/send/recv/close operations
//! - Session configuration for per-connection transforms
//! - State management for tracking data across operations

mod action;
mod context;
mod event;
mod pattern;
mod session;

use std::ffi::c_char;

/// Decode a `{ptr, len}` ABI string into `Option<&str>`.
///
/// Runtime callers populate these fields with `allocator.dupe` slices that are
/// not guaranteed to be NUL-terminated, so we must honor `len` instead of
/// scanning for a terminator.
pub(crate) fn str_opt_from_ptr_len<'a>(ptr: *const c_char, len: usize) -> Option<&'a str> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    std::str::from_utf8(bytes).ok()
}

/// Decode a `{ptr, len}` ABI string, returning an empty `&str` when missing or invalid UTF-8.
pub(crate) fn str_from_ptr_len<'a>(ptr: *const c_char, len: usize) -> &'a str {
    str_opt_from_ptr_len(ptr, len).unwrap_or("")
}

// Re-export all public types
pub use action::{
    AcceptResult, ConnectResult, NetAction, NetDirection, NetInspectionMode, NetPhaseAction,
};
pub use context::NetContext;
pub use event::{
    AcceptEvent, CloseEvent, ConnectEvent, DomainEvent, ProtocolEvent, RecvEvent, SendEvent,
    TlsEvent,
};
pub use pattern::NetPattern;
pub use session::{NetRwConfig, NetSession, NetSessionBuilder, NetTransformFn, SessionState};

// Re-export plugin state for convenience (used in callbacks)
pub use crate::state::{FileState, PluginState};

// Re-export Buffer from parent module for convenience
pub use crate::buffer::Buffer;

/// Callback type for connect events.
///
/// Receives the connect event and returns an action determining how to handle the connection.
pub type ConnectFn = fn(&ConnectEvent) -> ConnectResult;

/// Callback type for accept events.
///
/// Receives the accept event and returns an action determining how to handle the connection.
pub type AcceptFn = fn(&AcceptEvent) -> AcceptResult;

/// Callback type for TLS events.
///
/// Receives the session state and TLS event. Called when TLS handshake completes.
pub type TlsFn = fn(PluginState, &TlsEvent) -> NetPhaseAction;

/// Callback type for domain events.
///
/// Receives the session state and domain event. Called when domain is discovered.
pub type DomainFn = fn(PluginState, &DomainEvent) -> NetPhaseAction;

/// Callback type for protocol events.
///
/// Receives the session state and protocol event. Called when protocol is detected.
pub type ProtocolFn = fn(PluginState, &ProtocolEvent) -> NetPhaseAction;

/// Callback type for send events.
///
/// Receives the session state and send event, returns an action.
pub type SendFn = fn(PluginState, &SendEvent) -> NetAction;

/// Callback type for recv events.
///
/// Receives the session state and recv event, returns an action.
pub type RecvFn = fn(PluginState, &RecvEvent) -> NetAction;

/// Callback type for close events.
///
/// Receives the session state and close event. Called for cleanup.
pub type CloseFn = fn(PluginState, &CloseEvent);
