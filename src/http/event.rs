//! HTTP event types
//!
//! Lifetime-bound wrappers around C event structures.
//!
//! Primary accessors return `&[u8]` (byte slices) because HTTP fields are not
//! guaranteed to be valid UTF-8. Convenience `_str()` methods are provided
//! that perform checked UTF-8 conversion.

use crate::ffi;
use crate::http::context::{HttpCloseReason, HttpContext, HttpExchangeFlags, HttpMessageKind};

/// A single HTTP header (name-value pair).
#[derive(Debug, Clone, Copy)]
pub struct HttpHeader<'a> {
    inner: &'a ffi::qcontrol_http_header_t,
}

impl<'a> HttpHeader<'a> {
    /// Get the header name as raw bytes.
    pub fn name(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.name, self.inner.name_len)
    }

    /// Get the header name as a UTF-8 string, if valid.
    pub fn name_str(&self) -> Option<&str> {
        std::str::from_utf8(self.name()).ok()
    }

    /// Get the header value as raw bytes.
    pub fn value(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.value, self.inner.value_len)
    }

    /// Get the header value as a UTF-8 string, if valid.
    pub fn value_str(&self) -> Option<&str> {
        std::str::from_utf8(self.value()).ok()
    }
}

/// Read a length-delimited byte slice from an FFI struct field.
fn bytes_from_ptr_len(ptr: *const core::ffi::c_char, len: usize) -> &'static [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
    }
}

/// Helper to read headers from an FFI event.
unsafe fn headers_from_raw<'a>(
    ptr: *const ffi::qcontrol_http_header_t,
    count: usize,
) -> &'a [ffi::qcontrol_http_header_t] {
    if ptr.is_null() || count == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, count)
    }
}

/// Iterator over HTTP headers.
pub struct HttpHeaders<'a> {
    raw: &'a [ffi::qcontrol_http_header_t],
    pos: usize,
}

impl<'a> Iterator for HttpHeaders<'a> {
    type Item = HttpHeader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.raw.len() {
            let h = HttpHeader {
                inner: &self.raw[self.pos],
            };
            self.pos += 1;
            Some(h)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.raw.len() - self.pos;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for HttpHeaders<'a> {}

// ============================================================================
// Request Event
// ============================================================================

/// Event for HTTP request headers.
pub struct HttpRequestEvent<'a> {
    inner: &'a ffi::qcontrol_http_request_event_t,
}

impl<'a> HttpRequestEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_request_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the raw request target as seen on the wire.
    pub fn raw_target(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.raw_target, self.inner.raw_target_len)
    }

    /// Get the raw request target as a UTF-8 string, if valid.
    pub fn raw_target_str(&self) -> Option<&str> {
        std::str::from_utf8(self.raw_target()).ok()
    }

    /// Get the normalized request method.
    pub fn method(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.method, self.inner.method_len)
    }

    /// Get the normalized request method as a UTF-8 string, if valid.
    pub fn method_str(&self) -> Option<&str> {
        std::str::from_utf8(self.method()).ok()
    }

    /// Get the request scheme, if available.
    pub fn scheme(&self) -> Option<&[u8]> {
        let s = bytes_from_ptr_len(self.inner.scheme, self.inner.scheme_len);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Get the request authority, if available.
    pub fn authority(&self) -> Option<&[u8]> {
        let s = bytes_from_ptr_len(self.inner.authority, self.inner.authority_len);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Get the normalized request path.
    pub fn path(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.path, self.inner.path_len)
    }

    /// Get the normalized request path as a UTF-8 string, if valid.
    pub fn path_str(&self) -> Option<&str> {
        std::str::from_utf8(self.path()).ok()
    }

    /// Get the number of headers.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }

    /// Find a header value by name (case-insensitive ASCII comparison).
    pub fn header(&self, name: &[u8]) -> Option<&[u8]> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        for h in raw {
            let hname = bytes_from_ptr_len(h.name, h.name_len);
            if hname.eq_ignore_ascii_case(name) {
                return Some(bytes_from_ptr_len(h.value, h.value_len));
            }
        }
        None
    }
}

// ============================================================================
// Response Event
// ============================================================================

/// Event for HTTP response headers.
pub struct HttpResponseEvent<'a> {
    inner: &'a ffi::qcontrol_http_response_event_t,
}

impl<'a> HttpResponseEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_response_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the response status code.
    pub fn status_code(&self) -> u16 {
        self.inner.status_code
    }

    /// Get the reason phrase as raw bytes, if available.
    pub fn reason(&self) -> Option<&[u8]> {
        let s = bytes_from_ptr_len(self.inner.reason, self.inner.reason_len);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Get the reason phrase as a UTF-8 string, if valid and available.
    pub fn reason_str(&self) -> Option<&str> {
        self.reason().and_then(|b| std::str::from_utf8(b).ok())
    }

    /// Get the number of headers.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }

    /// Find a header value by name (case-insensitive ASCII comparison).
    pub fn header(&self, name: &[u8]) -> Option<&[u8]> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        for h in raw {
            let hname = bytes_from_ptr_len(h.name, h.name_len);
            if hname.eq_ignore_ascii_case(name) {
                return Some(bytes_from_ptr_len(h.value, h.value_len));
            }
        }
        None
    }
}

// ============================================================================
// Body Event
// ============================================================================

/// Body flag bits.
#[derive(Debug, Clone, Copy)]
pub struct HttpBodyFlags(u32);

impl HttpBodyFlags {
    /// Whether transfer framing has been decoded (e.g., chunked encoding).
    pub fn transfer_decoded(&self) -> bool {
        self.0 & ffi::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_TRANSFER_DECODED != 0
    }

    /// Whether content encoding has been decoded (e.g., gzip).
    pub fn content_decoded(&self) -> bool {
        self.0 & ffi::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_CONTENT_DECODED != 0
    }
}

/// Event for an HTTP body chunk (request or response).
pub struct HttpBodyEvent<'a> {
    inner: &'a ffi::qcontrol_http_body_event_t,
}

impl<'a> HttpBodyEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_body_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the message kind (request or response).
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Get the body bytes for this chunk.
    pub fn bytes(&self) -> &[u8] {
        if self.inner.bytes.is_null() || self.inner.bytes_len == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(self.inner.bytes as *const u8, self.inner.bytes_len)
            }
        }
    }

    /// Get the body bytes as a string if valid UTF-8.
    pub fn bytes_str(&self) -> Option<&str> {
        std::str::from_utf8(self.bytes()).ok()
    }

    /// Get the decoded body offset within this message.
    pub fn offset(&self) -> u64 {
        self.inner.offset
    }

    /// Get the body flags.
    pub fn flags(&self) -> HttpBodyFlags {
        HttpBodyFlags(self.inner.flags)
    }
}

// ============================================================================
// Trailers Event
// ============================================================================

/// Event for HTTP trailers (request or response).
pub struct HttpTrailersEvent<'a> {
    inner: &'a ffi::qcontrol_http_trailers_event_t,
}

impl<'a> HttpTrailersEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_trailers_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the message kind (request or response).
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Get the number of trailers.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over trailers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }
}

// ============================================================================
// Message Done Event
// ============================================================================

/// Event for HTTP message completion (request or response).
pub struct HttpMessageDoneEvent<'a> {
    inner: &'a ffi::qcontrol_http_message_done_event_t,
}

impl<'a> HttpMessageDoneEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_message_done_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the message kind (request or response).
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Get the total decoded body bytes observed for this message.
    pub fn body_bytes(&self) -> u64 {
        self.inner.body_bytes
    }
}

// ============================================================================
// Exchange Close Event
// ============================================================================

/// Event for HTTP exchange close.
pub struct HttpExchangeCloseEvent<'a> {
    inner: &'a ffi::qcontrol_http_exchange_close_event_t,
}

impl<'a> HttpExchangeCloseEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_exchange_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Get the close reason.
    pub fn reason(&self) -> HttpCloseReason {
        HttpCloseReason::from_ffi(self.inner.reason)
    }

    /// Get the exchange completion flags.
    pub fn flags(&self) -> HttpExchangeFlags {
        HttpExchangeFlags::from_raw(self.inner.flags)
    }
}
