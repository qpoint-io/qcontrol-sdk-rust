//! HTTP operation types
//!
//! This module provides types for HTTP exchange observation:
//! - Events for request/response headers, body chunks, trailers, and completion
//! - Action types for pass/block/state decisions
//! - HTTP context with version, exchange ID, and stream metadata
//! - State management for tracking data across an HTTP exchange

mod action;
mod context;
mod event;

// Re-export all public types
pub use action::{HttpAction, HttpRequestAction};
pub use context::HttpContext;
pub use event::{
    HttpBodyEvent, HttpBodyFlags, HttpExchangeCloseEvent, HttpHeader, HttpHeaders,
    HttpMessageDoneEvent, HttpRequestEvent, HttpResponseEvent, HttpTrailersEvent,
};

// Re-export enums
pub use context::{HttpCloseReason, HttpExchangeFlags, HttpMessageKind, HttpVersion};

// Re-export FileState for convenience (used in callbacks)
pub use crate::file::FileState;

/// State wrapper for HTTP exchange state.
///
/// Wraps user-provided state so it can be passed through the C ABI
/// and cleaned up on exchange close.
#[doc(hidden)]
pub struct HttpState {
    /// User-provided state (may be None).
    pub user_state: Option<Box<dyn std::any::Any + Send>>,
}

impl HttpState {
    /// Get a FileState referencing the user's state.
    pub fn as_file_state(&self) -> FileState<'_> {
        match &self.user_state {
            Some(boxed) => FileState::from_ref(boxed.as_ref()),
            None => FileState::empty(),
        }
    }
}

/// Callback type for HTTP request events (headers).
///
/// Called once per exchange when request headers arrive.
/// Returns an action: Pass, Block, or State (to attach per-exchange state).
pub type HttpRequestFn = fn(&HttpRequestEvent) -> HttpRequestAction;

/// Callback type for HTTP request body chunks.
///
/// Receives exchange state and a body chunk event. Returns an action.
pub type HttpRequestBodyFn = fn(FileState, &HttpBodyEvent) -> HttpAction;

/// Callback type for HTTP request trailers.
///
/// Receives exchange state and trailers event. Returns an action.
pub type HttpRequestTrailersFn = fn(FileState, &HttpTrailersEvent) -> HttpAction;

/// Callback type for HTTP request completion.
///
/// Receives exchange state and done event. Called when the request message is complete.
pub type HttpRequestDoneFn = fn(FileState, &HttpMessageDoneEvent);

/// Callback type for HTTP response events (headers).
///
/// Receives exchange state and response headers. Returns an action.
pub type HttpResponseFn = fn(FileState, &HttpResponseEvent) -> HttpAction;

/// Callback type for HTTP response body chunks.
///
/// Receives exchange state and a body chunk event. Returns an action.
pub type HttpResponseBodyFn = fn(FileState, &HttpBodyEvent) -> HttpAction;

/// Callback type for HTTP response trailers.
///
/// Receives exchange state and trailers event. Returns an action.
pub type HttpResponseTrailersFn = fn(FileState, &HttpTrailersEvent) -> HttpAction;

/// Callback type for HTTP response completion.
///
/// Receives exchange state and done event. Called when the response message is complete.
pub type HttpResponseDoneFn = fn(FileState, &HttpMessageDoneEvent);

/// Callback type for HTTP exchange close.
///
/// Called exactly once per tracked exchange, including abnormal termination.
/// Plugin should clean up any exchange state here.
pub type HttpExchangeCloseFn = fn(FileState, &HttpExchangeCloseEvent);
