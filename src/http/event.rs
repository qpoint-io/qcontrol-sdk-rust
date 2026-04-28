//! HTTP event types.
//!
//! This module wraps the C HTTP event structures in Rust types that expose the
//! repository's coalesced HTTP ABI. Request, response, body, and trailers
//! callbacks all use one event family, while mutation is surfaced through
//! optional host-backed handles embedded in those same events.

use std::ffi::c_char;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::ffi;
use crate::http::context::{HttpCloseReason, HttpContext, HttpExchangeFlags, HttpMessageKind};
use crate::{Buffer, BufferRef};

/// A single HTTP header (name-value pair).
#[derive(Debug, Clone, Copy)]
pub struct HttpHeader<'a> {
    inner: &'a ffi::qcontrol_http_header_t,
}

impl<'a> HttpHeader<'a> {
    /// Return the header name as raw bytes.
    pub fn name(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.name, self.inner.name_len)
    }

    /// Return the header name as UTF-8 when the bytes are valid text.
    pub fn name_str(&self) -> Option<&str> {
        std::str::from_utf8(self.name()).ok()
    }

    /// Return the header value as raw bytes.
    pub fn value(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.value, self.inner.value_len)
    }

    /// Return the header value as UTF-8 when the bytes are valid text.
    pub fn value_str(&self) -> Option<&str> {
        std::str::from_utf8(self.value()).ok()
    }
}

/// Error returned when the SDK cannot encode a JSON body replacement.
#[derive(Debug)]
pub enum HttpBodySetJsonError {
    /// The current host/path does not provide a mutable body buffer.
    MutationUnavailable,
    /// The replacement value could not be serialized to JSON.
    Serialize(serde_json::Error),
}

impl std::fmt::Display for HttpBodySetJsonError {
    /// Describe why JSON replacement could not be completed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpBodySetJsonError::MutationUnavailable => {
                f.write_str("HTTP body mutation is unavailable for this callback")
            }
            HttpBodySetJsonError::Serialize(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for HttpBodySetJsonError {
    /// Return the underlying serialization error when one exists.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HttpBodySetJsonError::MutationUnavailable => None,
            HttpBodySetJsonError::Serialize(err) => Some(err),
        }
    }
}

/// Read a length-delimited byte slice from an FFI struct field.
fn bytes_from_ptr_len(ptr: *const c_char, len: usize) -> &'static [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
    }
}

/// Read the current contents of a host-backed mutable body buffer.
fn bytes_from_buffer_ptr(ptr: *mut ffi::qcontrol_buffer_t) -> &'static [u8] {
    if ptr.is_null() {
        return &[];
    }

    unsafe {
        let data = ffi::qcontrol_buffer_data(ptr);
        let len = ffi::qcontrol_buffer_len(ptr);
        if data.is_null() || len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(data as *const u8, len)
        }
    }
}

/// View one contiguous header array returned by the runtime.
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

    /// Return the next header in the current header view.
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.raw.len() {
            let header = HttpHeader {
                inner: &self.raw[self.pos],
            };
            self.pos += 1;
            Some(header)
        } else {
            None
        }
    }

    /// Report the number of headers left in the iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.raw.len() - self.pos;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for HttpHeaders<'a> {}

/// Mutable view over one HTTP header block owned by the host.
pub struct HttpHeadersMut<'a> {
    inner: *mut ffi::qcontrol_http_headers_t,
    _marker: PhantomData<&'a mut ffi::qcontrol_http_headers_t>,
}

impl<'a> HttpHeadersMut<'a> {
    /// Construct a mutable header block from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_headers_t) -> Self {
        Self {
            inner: ptr,
            _marker: PhantomData,
        }
    }

    /// Return the number of headers currently in the block.
    pub fn count(&self) -> usize {
        unsafe { ffi::qcontrol_http_headers_count(self.inner) }
    }

    /// Return whether the current block contains no headers.
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Iterate over the current header view.
    pub fn iter(&self) -> HttpHeaders<'_> {
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(self.inner),
                ffi::qcontrol_http_headers_count(self.inner),
            )
        };
        HttpHeaders { raw, pos: 0 }
    }

    /// Find the first header value whose name matches case-insensitively.
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(self.inner),
                ffi::qcontrol_http_headers_count(self.inner),
            )
        };
        for header in raw {
            let header_name = bytes_from_ptr_len(header.name, header.name_len);
            if header_name.eq_ignore_ascii_case(name) {
                return Some(bytes_from_ptr_len(header.value, header.value_len));
            }
        }
        None
    }

    /// Find the first header value by ASCII name when the value is valid UTF-8.
    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.get(name.as_bytes())
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Append one header without removing existing headers of the same name.
    pub fn add(&mut self, name: &[u8], value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_headers_add(
                self.inner,
                name.as_ptr() as *const c_char,
                name.len(),
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Append one header from UTF-8 strings.
    pub fn add_str(&mut self, name: &str, value: &str) -> bool {
        self.add(name.as_bytes(), value.as_bytes())
    }

    /// Replace all headers with the same name with one new header value.
    pub fn set(&mut self, name: &[u8], value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_headers_set(
                self.inner,
                name.as_ptr() as *const c_char,
                name.len(),
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace one header using UTF-8 strings.
    pub fn set_str(&mut self, name: &str, value: &str) -> bool {
        self.set(name.as_bytes(), value.as_bytes())
    }

    /// Remove every header whose name matches case-insensitively.
    pub fn remove(&mut self, name: &[u8]) -> usize {
        unsafe {
            ffi::qcontrol_http_headers_remove(
                self.inner,
                name.as_ptr() as *const c_char,
                name.len(),
            )
        }
    }

    /// Remove every header with the given UTF-8 name.
    pub fn remove_str(&mut self, name: &str) -> usize {
        self.remove(name.as_bytes())
    }
}

/// Mutable request head handle supplied by hosts that support head editing.
pub struct HttpRequestHead<'a> {
    inner: *mut ffi::qcontrol_http_request_head_t,
    _marker: PhantomData<&'a mut ffi::qcontrol_http_request_head_t>,
}

impl<'a> HttpRequestHead<'a> {
    /// Construct a request head wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_request_head_t) -> Self {
        Self {
            inner: ptr,
            _marker: PhantomData,
        }
    }

    /// Return the raw request-target bytes.
    pub fn raw_target(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_raw_target(self.inner),
                ffi::qcontrol_http_request_raw_target_len(self.inner),
            )
        }
    }

    /// Return the raw request-target as UTF-8 when valid.
    pub fn raw_target_str(&self) -> Option<&str> {
        std::str::from_utf8(self.raw_target()).ok()
    }

    /// Return the normalized request method.
    pub fn method(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_method(self.inner),
                ffi::qcontrol_http_request_method_len(self.inner),
            )
        }
    }

    /// Return the normalized request method as UTF-8 when valid.
    pub fn method_str(&self) -> Option<&str> {
        std::str::from_utf8(self.method()).ok()
    }

    /// Replace the request method.
    pub fn set_method(&mut self, value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_request_set_method(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the request method from a UTF-8 string.
    pub fn set_method_str(&mut self, value: &str) -> bool {
        self.set_method(value.as_bytes())
    }

    /// Return the request scheme when one is available.
    pub fn scheme(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_scheme(self.inner),
                ffi::qcontrol_http_request_scheme_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the request scheme as UTF-8 when one is available and valid.
    pub fn scheme_str(&self) -> Option<&str> {
        self.scheme()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Replace the request scheme.
    pub fn set_scheme(&mut self, value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_request_set_scheme(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the request scheme from a UTF-8 string.
    pub fn set_scheme_str(&mut self, value: &str) -> bool {
        self.set_scheme(value.as_bytes())
    }

    /// Return the request authority when one is available.
    pub fn authority(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_authority(self.inner),
                ffi::qcontrol_http_request_authority_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the request authority as UTF-8 when one is available and valid.
    pub fn authority_str(&self) -> Option<&str> {
        self.authority()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Replace the request authority.
    pub fn set_authority(&mut self, value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_request_set_authority(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the request authority from a UTF-8 string.
    pub fn set_authority_str(&mut self, value: &str) -> bool {
        self.set_authority(value.as_bytes())
    }

    /// Return the normalized request path.
    pub fn path(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_path(self.inner),
                ffi::qcontrol_http_request_path_len(self.inner),
            )
        }
    }

    /// Return the normalized request path as UTF-8 when valid.
    pub fn path_str(&self) -> Option<&str> {
        std::str::from_utf8(self.path()).ok()
    }

    /// Replace the normalized request path.
    pub fn set_path(&mut self, value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_request_set_path(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the normalized request path from a UTF-8 string.
    pub fn set_path_str(&mut self, value: &str) -> bool {
        self.set_path(value.as_bytes())
    }

    /// Iterate over the current request headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let headers = unsafe { ffi::qcontrol_http_request_headers(self.inner) };
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(headers),
                ffi::qcontrol_http_headers_count(headers),
            )
        };
        HttpHeaders { raw, pos: 0 }
    }

    /// Return the mutable request header block.
    pub fn headers_mut(&mut self) -> HttpHeadersMut<'_> {
        unsafe { HttpHeadersMut::from_raw(ffi::qcontrol_http_request_headers(self.inner)) }
    }
}

/// Read-only request head handle supplied alongside [`HttpRequestHead`].
pub struct HttpRequestHeadRef<'a> {
    inner: *const ffi::qcontrol_http_request_head_t,
    _marker: PhantomData<&'a ffi::qcontrol_http_request_head_t>,
}

impl<'a> HttpRequestHeadRef<'a> {
    /// Construct a read-only request head wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *const ffi::qcontrol_http_request_head_t) -> Self {
        Self {
            inner: ptr,
            _marker: PhantomData,
        }
    }

    /// Return the raw request-target bytes.
    pub fn raw_target(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_raw_target(self.inner),
                ffi::qcontrol_http_request_raw_target_len(self.inner),
            )
        }
    }

    /// Return the raw request-target as UTF-8 when valid.
    pub fn raw_target_str(&self) -> Option<&str> {
        std::str::from_utf8(self.raw_target()).ok()
    }

    /// Return the normalized request method.
    pub fn method(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_method(self.inner),
                ffi::qcontrol_http_request_method_len(self.inner),
            )
        }
    }

    /// Return the normalized request method as UTF-8 when valid.
    pub fn method_str(&self) -> Option<&str> {
        std::str::from_utf8(self.method()).ok()
    }

    /// Return the request scheme when one is available.
    pub fn scheme(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_scheme(self.inner),
                ffi::qcontrol_http_request_scheme_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the request scheme as UTF-8 when one is available and valid.
    pub fn scheme_str(&self) -> Option<&str> {
        self.scheme()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the request authority when one is available.
    pub fn authority(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_authority(self.inner),
                ffi::qcontrol_http_request_authority_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the request authority as UTF-8 when one is available and valid.
    pub fn authority_str(&self) -> Option<&str> {
        self.authority()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the normalized request path.
    pub fn path(&self) -> &[u8] {
        unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_request_path(self.inner),
                ffi::qcontrol_http_request_path_len(self.inner),
            )
        }
    }

    /// Return the normalized request path as UTF-8 when valid.
    pub fn path_str(&self) -> Option<&str> {
        std::str::from_utf8(self.path()).ok()
    }

    /// Iterate over the current request headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let headers = unsafe { ffi::qcontrol_http_request_headers(self.inner as *mut _) };
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(headers),
                ffi::qcontrol_http_headers_count(headers),
            )
        };
        HttpHeaders { raw, pos: 0 }
    }
}

/// Mutable response head handle supplied by hosts that support head editing.
pub struct HttpResponseHead<'a> {
    inner: *mut ffi::qcontrol_http_response_head_t,
    _marker: PhantomData<&'a mut ffi::qcontrol_http_response_head_t>,
}

/// Read-only response head handle supplied alongside [`HttpResponseHead`].
pub struct HttpResponseHeadRef<'a> {
    inner: *const ffi::qcontrol_http_response_head_t,
    _marker: PhantomData<&'a ffi::qcontrol_http_response_head_t>,
}

impl<'a> HttpResponseHeadRef<'a> {
    /// Construct a read-only response head wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *const ffi::qcontrol_http_response_head_t) -> Self {
        Self {
            inner: ptr,
            _marker: PhantomData,
        }
    }

    /// Return the response status code.
    pub fn status_code(&self) -> u16 {
        unsafe { ffi::qcontrol_http_response_status_code(self.inner) }
    }

    /// Return the reason phrase when one is available.
    pub fn reason(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_response_reason(self.inner),
                ffi::qcontrol_http_response_reason_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the reason phrase as UTF-8 when one is available and valid.
    pub fn reason_str(&self) -> Option<&str> {
        self.reason()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Iterate over the current response headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let headers = unsafe { ffi::qcontrol_http_response_headers(self.inner as *mut _) };
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(headers),
                ffi::qcontrol_http_headers_count(headers),
            )
        };
        HttpHeaders { raw, pos: 0 }
    }
}

impl<'a> HttpResponseHead<'a> {
    /// Construct a response head wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_response_head_t) -> Self {
        Self {
            inner: ptr,
            _marker: PhantomData,
        }
    }

    /// Return the response status code.
    pub fn status_code(&self) -> u16 {
        unsafe { ffi::qcontrol_http_response_status_code(self.inner) }
    }

    /// Replace the response status code.
    pub fn set_status_code(&mut self, status_code: u16) {
        unsafe {
            ffi::qcontrol_http_response_set_status_code(self.inner, status_code);
        }
    }

    /// Return the reason phrase when one is available.
    pub fn reason(&self) -> Option<&[u8]> {
        let bytes = unsafe {
            bytes_from_ptr_len(
                ffi::qcontrol_http_response_reason(self.inner),
                ffi::qcontrol_http_response_reason_len(self.inner),
            )
        };
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    }

    /// Return the reason phrase as UTF-8 when one is available and valid.
    pub fn reason_str(&self) -> Option<&str> {
        self.reason()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Replace the response reason phrase.
    pub fn set_reason(&mut self, value: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_http_response_set_reason(
                self.inner,
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the response reason phrase from a UTF-8 string.
    pub fn set_reason_str(&mut self, value: &str) -> bool {
        self.set_reason(value.as_bytes())
    }

    /// Iterate over the current response headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let headers = unsafe { ffi::qcontrol_http_response_headers(self.inner) };
        let raw = unsafe {
            headers_from_raw(
                ffi::qcontrol_http_headers_data(headers),
                ffi::qcontrol_http_headers_count(headers),
            )
        };
        HttpHeaders { raw, pos: 0 }
    }

    /// Return the mutable response header block.
    pub fn headers_mut(&mut self) -> HttpHeadersMut<'_> {
        unsafe { HttpHeadersMut::from_raw(ffi::qcontrol_http_response_headers(self.inner)) }
    }
}

/// Event for HTTP request headers.
pub struct HttpRequestEvent<'a> {
    inner: &'a mut ffi::qcontrol_http_request_event_t,
}

impl<'a> HttpRequestEvent<'a> {
    /// Construct a request event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_request_event_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return the raw request-target as seen on the wire.
    pub fn raw_target(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.raw_target, self.inner.raw_target_len)
    }

    /// Return the raw request-target as UTF-8 when valid.
    pub fn raw_target_str(&self) -> Option<&str> {
        std::str::from_utf8(self.raw_target()).ok()
    }

    /// Return the normalized request method.
    pub fn method(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.method, self.inner.method_len)
    }

    /// Return the normalized request method as UTF-8 when valid.
    pub fn method_str(&self) -> Option<&str> {
        std::str::from_utf8(self.method()).ok()
    }

    /// Return the request scheme when one is available.
    pub fn scheme(&self) -> Option<&[u8]> {
        let value = bytes_from_ptr_len(self.inner.scheme, self.inner.scheme_len);
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    /// Return the request scheme as UTF-8 when one is available and valid.
    pub fn scheme_str(&self) -> Option<&str> {
        self.scheme()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the request authority when one is available.
    pub fn authority(&self) -> Option<&[u8]> {
        let value = bytes_from_ptr_len(self.inner.authority, self.inner.authority_len);
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    /// Return the request authority as UTF-8 when one is available and valid.
    pub fn authority_str(&self) -> Option<&str> {
        self.authority()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the normalized request path.
    pub fn path(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.path, self.inner.path_len)
    }

    /// Return the normalized request path as UTF-8 when valid.
    pub fn path_str(&self) -> Option<&str> {
        std::str::from_utf8(self.path()).ok()
    }

    /// Return the number of request headers.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over the request headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }

    /// Find the first request header value by ASCII name.
    pub fn header(&self, name: &[u8]) -> Option<&[u8]> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        for header in raw {
            let header_name = bytes_from_ptr_len(header.name, header.name_len);
            if header_name.eq_ignore_ascii_case(name) {
                return Some(bytes_from_ptr_len(header.value, header.value_len));
            }
        }
        None
    }

    /// Return the read-only request head handle when the host supports it.
    pub fn head(&self) -> Option<HttpRequestHeadRef<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe { HttpRequestHeadRef::from_raw(self.inner.head) })
        }
    }

    /// Return the mutable request head handle when the host supports it.
    pub fn head_mut(&mut self) -> Option<HttpRequestHead<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe { HttpRequestHead::from_raw(self.inner.head) })
        }
    }

    /// Return the mutable request header block when the host supports it.
    pub fn headers_mut(&mut self) -> Option<HttpHeadersMut<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe {
                HttpHeadersMut::from_raw(ffi::qcontrol_http_request_headers(self.inner.head))
            })
        }
    }
}

/// Event for HTTP response headers.
pub struct HttpResponseEvent<'a> {
    inner: &'a mut ffi::qcontrol_http_response_event_t,
}

impl<'a> HttpResponseEvent<'a> {
    /// Construct a response event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_response_event_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return the current response status code.
    pub fn status_code(&self) -> u16 {
        self.inner.status_code
    }

    /// Return the response reason phrase when one is available.
    pub fn reason(&self) -> Option<&[u8]> {
        let value = bytes_from_ptr_len(self.inner.reason, self.inner.reason_len);
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }

    /// Return the response reason phrase as UTF-8 when valid.
    pub fn reason_str(&self) -> Option<&str> {
        self.reason()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the number of response headers.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over the response headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }

    /// Find the first response header value by ASCII name.
    pub fn header(&self, name: &[u8]) -> Option<&[u8]> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        for header in raw {
            let header_name = bytes_from_ptr_len(header.name, header.name_len);
            if header_name.eq_ignore_ascii_case(name) {
                return Some(bytes_from_ptr_len(header.value, header.value_len));
            }
        }
        None
    }

    /// Return the read-only response head handle when the host supports it.
    pub fn head(&self) -> Option<HttpResponseHeadRef<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe { HttpResponseHeadRef::from_raw(self.inner.head) })
        }
    }

    /// Return the mutable response head handle when the host supports it.
    pub fn head_mut(&mut self) -> Option<HttpResponseHead<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe { HttpResponseHead::from_raw(self.inner.head) })
        }
    }

    /// Return the mutable response header block when the host supports it.
    pub fn headers_mut(&mut self) -> Option<HttpHeadersMut<'_>> {
        if self.inner.head.is_null() {
            None
        } else {
            Some(unsafe {
                HttpHeadersMut::from_raw(ffi::qcontrol_http_response_headers(self.inner.head))
            })
        }
    }
}

/// Body flag bits.
#[derive(Debug, Clone, Copy)]
pub struct HttpBodyFlags(u32);

impl HttpBodyFlags {
    /// Report whether transfer framing has already been removed.
    pub fn transfer_decoded(&self) -> bool {
        self.0 & ffi::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_TRANSFER_DECODED != 0
    }

    /// Report whether content encoding has already been removed.
    pub fn content_decoded(&self) -> bool {
        self.0 & ffi::qcontrol_http_body_flag_t_QCONTROL_HTTP_BODY_FLAG_CONTENT_DECODED != 0
    }
}

/// Event for an HTTP body callback.
pub struct HttpBodyEvent<'a> {
    inner: &'a mut ffi::qcontrol_http_body_event_t,
}

impl<'a> HttpBodyEvent<'a> {
    /// Construct a body event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_body_event_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return whether this body callback belongs to a request or response.
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Return the decoded input bytes for this body callback.
    pub fn bytes(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.bytes, self.inner.bytes_len)
    }

    /// Return the decoded input bytes as UTF-8 when valid.
    pub fn bytes_str(&self) -> Option<&str> {
        std::str::from_utf8(self.bytes()).ok()
    }

    /// Return the current mutable output buffer contents when one exists.
    pub fn body_bytes(&self) -> Option<&[u8]> {
        if self.inner.body.is_null() {
            None
        } else {
            Some(bytes_from_buffer_ptr(self.inner.body))
        }
    }

    /// Return the mutable output buffer as UTF-8 when it exists and is valid.
    pub fn body_str(&self) -> Option<&str> {
        self.body_bytes()
            .and_then(|value| std::str::from_utf8(value).ok())
    }

    /// Return the host-backed read-only output buffer when one exists.
    pub fn body(&self) -> Option<BufferRef<'_>> {
        if self.inner.body.is_null() {
            None
        } else {
            Some(unsafe { BufferRef::from_raw(self.inner.body) })
        }
    }

    /// Return the host-backed mutable output buffer when one exists.
    pub fn body_mut(&mut self) -> Option<Buffer<'_>> {
        if self.inner.body.is_null() {
            None
        } else {
            Some(unsafe { Buffer::from_raw(self.inner.body) })
        }
    }

    /// Decode the current body input as JSON.
    ///
    /// Buffered-body hosts may populate the mutable body buffer with the full
    /// logical body. Other hosts may expose only the current input bytes.
    pub fn body_json<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        if let Some(bytes) = self.body_bytes() {
            serde_json::from_slice(bytes)
        } else {
            serde_json::from_slice(self.bytes())
        }
    }

    /// Serialize one value as JSON and replace the mutable body buffer.
    pub fn set_body_json<T: Serialize>(&mut self, value: &T) -> Result<(), HttpBodySetJsonError> {
        let encoded = serde_json::to_vec(value).map_err(HttpBodySetJsonError::Serialize)?;
        let mut body = self
            .body_mut()
            .ok_or(HttpBodySetJsonError::MutationUnavailable)?;
        body.set(&encoded);
        Ok(())
    }

    /// Return the decoded offset within the current message body.
    pub fn offset(&self) -> u64 {
        self.inner.offset
    }

    /// Return the body flags supplied by the host.
    pub fn flags(&self) -> HttpBodyFlags {
        HttpBodyFlags(self.inner.flags)
    }

    /// Return whether this callback carries the final body bytes.
    pub fn end_of_stream(&self) -> bool {
        self.inner.end_of_stream != 0
    }
}

/// Event for request or response trailers.
pub struct HttpTrailersEvent<'a> {
    inner: &'a mut ffi::qcontrol_http_trailers_event_t,
}

impl<'a> HttpTrailersEvent<'a> {
    /// Construct a trailers event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_trailers_event_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return whether these trailers belong to a request or response.
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Return the number of trailers in the current block.
    pub fn header_count(&self) -> usize {
        self.inner.header_count
    }

    /// Iterate over the trailer headers.
    pub fn headers(&self) -> HttpHeaders<'_> {
        let raw = unsafe { headers_from_raw(self.inner.headers, self.inner.header_count) };
        HttpHeaders { raw, pos: 0 }
    }

    /// Return the mutable trailer block when the host supports it.
    pub fn headers_mut(&mut self) -> Option<HttpHeadersMut<'_>> {
        if self.inner.header_block.is_null() {
            None
        } else {
            Some(unsafe { HttpHeadersMut::from_raw(self.inner.header_block) })
        }
    }
}

/// Event for HTTP message completion (request or response).
pub struct HttpMessageDoneEvent<'a> {
    inner: &'a ffi::qcontrol_http_message_done_event_t,
}

impl<'a> HttpMessageDoneEvent<'a> {
    /// Construct a message-done event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_message_done_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return whether the completed message was a request or response.
    pub fn kind(&self) -> HttpMessageKind {
        HttpMessageKind::from_ffi(self.inner.kind)
    }

    /// Return the total decoded body bytes observed for this message.
    pub fn body_bytes(&self) -> u64 {
        self.inner.body_bytes
    }
}

/// Event for HTTP exchange close.
pub struct HttpExchangeCloseEvent<'a> {
    inner: &'a ffi::qcontrol_http_exchange_close_event_t,
}

impl<'a> HttpExchangeCloseEvent<'a> {
    /// Construct an exchange-close event wrapper from the raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_http_exchange_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the HTTP context for this exchange.
    pub fn ctx(&self) -> HttpContext<'_> {
        unsafe { HttpContext::from_ref(&self.inner.ctx) }
    }

    /// Return the terminal exchange-close reason.
    pub fn reason(&self) -> HttpCloseReason {
        HttpCloseReason::from_ffi(self.inner.reason)
    }

    /// Return the exchange completion flags.
    pub fn flags(&self) -> HttpExchangeFlags {
        HttpExchangeFlags::from_raw(self.inner.flags)
    }
}
