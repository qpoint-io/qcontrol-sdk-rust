//! HTTP context and enum types
//!
//! Provides metadata about the HTTP exchange.

use crate::ffi;
use crate::net::NetContext;

/// HTTP context shared by all callbacks for one exchange.
///
/// Contains exchange metadata: version, IDs, and the underlying network context.
pub struct HttpContext<'a> {
    inner: &'a ffi::qcontrol_http_ctx_t,
}

impl<'a> HttpContext<'a> {
    /// Create from a reference to the FFI context (embedded in an event struct).
    #[doc(hidden)]
    pub unsafe fn from_ref(ctx: &'a ffi::qcontrol_http_ctx_t) -> Self {
        Self { inner: ctx }
    }

    /// Get the runtime-assigned exchange identifier.
    pub fn exchange_id(&self) -> u64 {
        self.inner.exchange_id
    }

    /// Get the HTTP/2 stream ID, or `None` for HTTP/1.x.
    pub fn stream_id(&self) -> Option<u32> {
        match self.version() {
            HttpVersion::Http2 => Some(self.inner.stream_id),
            _ => None,
        }
    }

    /// Get the HTTP version.
    pub fn version(&self) -> HttpVersion {
        HttpVersion::from_ffi(self.inner.version)
    }

    /// Get the underlying network context.
    pub fn net(&self) -> NetContext<'a> {
        unsafe { NetContext::from_ref(&self.inner.net) }
    }
}

/// Normalized HTTP version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Unknown,
    Http10,
    Http11,
    Http2,
}

impl HttpVersion {
    pub(crate) fn from_ffi(v: ffi::qcontrol_http_version_t) -> Self {
        match v {
            ffi::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_1_0 => HttpVersion::Http10,
            ffi::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_1_1 => HttpVersion::Http11,
            ffi::qcontrol_http_version_t_QCONTROL_HTTP_VERSION_2 => HttpVersion::Http2,
            _ => HttpVersion::Unknown,
        }
    }
}

/// Message kind within an HTTP exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMessageKind {
    Request,
    Response,
}

impl HttpMessageKind {
    pub(crate) fn from_ffi(k: ffi::qcontrol_http_message_kind_t) -> Self {
        match k {
            ffi::qcontrol_http_message_kind_t_QCONTROL_HTTP_MESSAGE_RESPONSE => {
                HttpMessageKind::Response
            }
            _ => HttpMessageKind::Request,
        }
    }
}

/// Exchange close reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpCloseReason {
    /// Exchange completed normally.
    Complete,
    /// Exchange ended before protocol completion.
    Aborted,
    /// Streamer disabled after parse or desync failure.
    ParseError,
    /// Underlying connection closed before exchange completion.
    ConnectionClosed,
}

impl HttpCloseReason {
    pub(crate) fn from_ffi(r: ffi::qcontrol_http_close_reason_t) -> Self {
        match r {
            ffi::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_ABORTED => {
                HttpCloseReason::Aborted
            }
            ffi::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_PARSE_ERROR => {
                HttpCloseReason::ParseError
            }
            ffi::qcontrol_http_close_reason_t_QCONTROL_HTTP_CLOSE_CONNECTION_CLOSED => {
                HttpCloseReason::ConnectionClosed
            }
            _ => HttpCloseReason::Complete,
        }
    }
}

/// Exchange completion flags.
#[derive(Debug, Clone, Copy)]
pub struct HttpExchangeFlags(u32);

impl HttpExchangeFlags {
    pub(crate) fn from_raw(flags: u32) -> Self {
        Self(flags)
    }

    /// Whether the request message reached its done callback.
    pub fn request_done(&self) -> bool {
        self.0 & ffi::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_REQUEST_DONE != 0
    }

    /// Whether the response message reached its done callback.
    pub fn response_done(&self) -> bool {
        self.0 & ffi::qcontrol_http_exchange_flag_t_QCONTROL_HTTP_EXCHANGE_FLAG_RESPONSE_DONE != 0
    }
}
