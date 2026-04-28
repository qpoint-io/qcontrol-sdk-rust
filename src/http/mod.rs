//! HTTP operation types.
//!
//! This module provides the Rust-facing HTTP plugin surface over the shared C
//! ABI. The callback family is exchange-based and supports both read-only
//! inspection and host-backed mutation through the same event wrappers.

mod action;
mod context;
mod event;

// Re-export all public types
pub use action::{HttpAction, HttpBodyMode, HttpRequestAction};
pub use context::HttpContext;
pub use event::{
    HttpBodyEvent, HttpBodyFlags, HttpBodySetJsonError, HttpExchangeCloseEvent, HttpHeader,
    HttpHeaders, HttpHeadersMut, HttpMessageDoneEvent, HttpRequestEvent, HttpRequestHead,
    HttpRequestHeadRef, HttpResponseEvent, HttpResponseHead, HttpResponseHeadRef,
    HttpTrailersEvent,
};

// Re-export enums
pub use context::{HttpCloseReason, HttpExchangeFlags, HttpMessageKind, HttpVersion};

// Re-export plugin state for convenience (used in callbacks)
pub use crate::state::{FileState, PluginState};

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
    /// Get a PluginState referencing the user's state.
    pub fn as_file_state(&self) -> PluginState<'_> {
        match &self.user_state {
            Some(boxed) => PluginState::from_ref(boxed.as_ref()),
            None => PluginState::empty(),
        }
    }
}

/// Callback type for HTTP request events (headers).
///
/// Called once per exchange when request headers arrive. The mutable event can
/// expose a host-backed request head and header block when the host supports
/// structured request mutation.
pub type HttpRequestFn = fn(&mut HttpRequestEvent) -> HttpRequestAction;

/// Callback type for HTTP request body callbacks.
///
/// Receives exchange state and a mutable body event. Hosts that support body
/// editing provide a non-NULL mutable buffer through the event wrapper.
pub type HttpRequestBodyFn = fn(PluginState, &mut HttpBodyEvent) -> HttpAction;

/// Callback type for HTTP request trailers.
///
/// Receives exchange state and a mutable trailers event. Hosts that support
/// trailer editing provide a mutable trailer block through the event wrapper.
pub type HttpRequestTrailersFn = fn(PluginState, &mut HttpTrailersEvent) -> HttpAction;

/// Callback type for HTTP request completion.
///
/// Receives exchange state and done event. Called when the request message is complete.
pub type HttpRequestDoneFn = fn(PluginState, &HttpMessageDoneEvent);

/// Callback type for HTTP response events (headers).
///
/// Receives exchange state and a mutable response event. Hosts that support
/// structured response mutation provide a mutable response head and header
/// block through the event wrapper.
pub type HttpResponseFn = fn(PluginState, &mut HttpResponseEvent) -> HttpAction;

/// Callback type for HTTP response body callbacks.
///
/// Receives exchange state and a mutable body event. Hosts that support body
/// editing provide a non-NULL mutable buffer through the event wrapper.
pub type HttpResponseBodyFn = fn(PluginState, &mut HttpBodyEvent) -> HttpAction;

/// Callback type for HTTP response trailers.
///
/// Receives exchange state and a mutable trailers event. Hosts that support
/// trailer editing provide a mutable trailer block through the event wrapper.
pub type HttpResponseTrailersFn = fn(PluginState, &mut HttpTrailersEvent) -> HttpAction;

/// Callback type for HTTP response completion.
///
/// Receives exchange state and done event. Called when the response message is complete.
pub type HttpResponseDoneFn = fn(PluginState, &HttpMessageDoneEvent);

/// Callback type for HTTP exchange close.
///
/// Called exactly once per tracked exchange, including abnormal termination.
/// Plugin should clean up any exchange state here.
pub type HttpExchangeCloseFn = fn(PluginState, &HttpExchangeCloseEvent);
