//! WebSocket event wrappers.
//!
//! This module exposes the Rust-facing wrappers for the post-upgrade
//! WebSocket callback family.

use std::borrow::Cow;
use std::ffi::c_char;

use crate::buffer::Buffer;
use crate::ffi;
use crate::stream::{StreamContext, StreamDirection};

/// Read one length-delimited byte slice from an FFI event field.
fn bytes_from_ptr_len(ptr: *const c_char, len: usize) -> &'static [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
    }
}

/// Logical WebSocket message kind delivered by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketMessageKind {
    Text,
    Binary,
}

impl From<ffi::qcontrol_websocket_message_type_t> for WebSocketMessageKind {
    fn from(kind: ffi::qcontrol_websocket_message_type_t) -> Self {
        if kind == ffi::qcontrol_websocket_message_type_t_QCONTROL_WEBSOCKET_MESSAGE_BINARY {
            WebSocketMessageKind::Binary
        } else {
            WebSocketMessageKind::Text
        }
    }
}

/// Event delivered when one HTTP upgrade handshake becomes a WebSocket stream.
pub struct WebSocketOpenEvent<'a> {
    inner: &'a ffi::qcontrol_websocket_open_event_t,
}

impl<'a> WebSocketOpenEvent<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_websocket_open_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'a> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return the negotiated subprotocol, or an empty slice when none exists.
    pub fn subprotocol(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.subprotocol, self.inner.subprotocol_len)
    }

    /// Return the negotiated extensions header value, or an empty slice when none exists.
    pub fn extensions(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.extensions, self.inner.extensions_len)
    }
}

/// Event delivered for one logical WebSocket message.
pub struct WebSocketMessageEvent<'a> {
    inner: &'a mut ffi::qcontrol_websocket_message_event_t,
}

impl<'a> WebSocketMessageEvent<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_websocket_message_event_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'_> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return the logical message kind.
    pub fn kind(&self) -> WebSocketMessageKind {
        self.inner.kind.into()
    }

    /// Return the logical message payload bytes.
    pub fn bytes(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.bytes, self.inner.bytes_len)
    }

    /// Return the logical message payload as UTF-8 text when this message is text.
    pub fn text(&self) -> Option<&str> {
        if self.kind() != WebSocketMessageKind::Text {
            return None;
        }
        std::str::from_utf8(self.bytes()).ok()
    }

    /// Return the logical message payload as lossy UTF-8 text when this message is text.
    pub fn text_lossy(&self) -> Option<Cow<'_, str>> {
        if self.kind() != WebSocketMessageKind::Text {
            return None;
        }
        Some(String::from_utf8_lossy(self.bytes()))
    }

    /// Return whether this message supports host-backed mutation.
    pub fn can_mutate(&self) -> bool {
        !self.inner.body.is_null()
    }

    /// Return the host-backed mutable message buffer when available.
    pub fn body_mut(&mut self) -> Option<Buffer<'_>> {
        if self.inner.body.is_null() {
            None
        } else {
            Some(unsafe { Buffer::from_raw(self.inner.body) })
        }
    }

    /// Replace the logical message payload bytes when host-backed mutation is available.
    pub fn set_bytes(&mut self, value: &[u8]) -> bool {
        let Some(mut body) = self.body_mut() else {
            return false;
        };
        body.set(value);
        true
    }

    /// Replace the logical message payload with UTF-8 text when mutation is available.
    pub fn set_text(&mut self, value: &str) -> bool {
        self.set_bytes(value.as_bytes())
    }
}

/// Event delivered when the upgraded WebSocket stream closes.
pub struct WebSocketCloseEvent<'a> {
    inner: &'a ffi::qcontrol_websocket_close_event_t,
}

impl<'a> WebSocketCloseEvent<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_websocket_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'a> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return which side initiated the close handshake.
    pub fn direction(&self) -> StreamDirection {
        self.inner.direction.into()
    }

    /// Return the observed close code, or zero when none was carried.
    pub fn code(&self) -> u16 {
        self.inner.code
    }

    /// Return the observed close reason, or an empty slice when none was carried.
    pub fn reason(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.reason, self.inner.reason_len)
    }

    /// Return whether the close handshake completed cleanly.
    pub fn clean(&self) -> bool {
        self.inner.clean != 0
    }
}
