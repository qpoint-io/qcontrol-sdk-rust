//! SSE event wrappers.
//!
//! This module exposes one Rust-facing view over the parsed SSE callback
//! records emitted by the runtime.

use std::ffi::c_char;
use std::str;

use crate::ffi;
use crate::http::HttpCloseReason;
use crate::stream::StreamContext;

/// Read one length-delimited byte slice from an FFI event field.
fn bytes_from_ptr_len(ptr: *const c_char, len: usize) -> &'static [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
    }
}

/// Return the raw mutable SSE event handle when the host supports event edits.
fn event_handle(raw: &ffi::qcontrol_sse_event_record_t) -> *mut ffi::qcontrol_sse_event_t {
    raw.event_handle
}

/// Return the host-backed SSE event ops table when mutation is supported.
fn event_ops(raw: &ffi::qcontrol_sse_event_record_t) -> Option<&ffi::qcontrol_sse_event_ops_t> {
    unsafe { event_handle(raw).as_ref()?.ops.as_ref() }
}

/// Return the host-backed mutable SSE event context when mutation is supported.
fn event_context(raw: &ffi::qcontrol_sse_event_record_t) -> *mut std::ffi::c_void {
    unsafe {
        event_handle(raw)
            .as_ref()
            .map(|handle| handle.context)
            .unwrap_or(std::ptr::null_mut())
    }
}

/// Read one host-backed optional byte slice from the mutable SSE event handle.
fn event_bytes(
    raw: &ffi::qcontrol_sse_event_record_t,
    data: impl FnOnce(
        &ffi::qcontrol_sse_event_ops_t,
    ) -> Option<unsafe extern "C" fn(*mut std::ffi::c_void) -> *const c_char>,
    len: impl FnOnce(
        &ffi::qcontrol_sse_event_ops_t,
    ) -> Option<unsafe extern "C" fn(*mut std::ffi::c_void) -> usize>,
) -> Option<&'static [u8]> {
    let ops = event_ops(raw)?;
    let data_fn = data(ops)?;
    let len_fn = len(ops)?;
    let context = event_context(raw);
    Some(unsafe { bytes_from_ptr_len(data_fn(context), len_fn(context)) })
}

/// Event delivered when one HTTP response starts an SSE stream.
pub struct SseOpenEvent<'a> {
    inner: &'a ffi::qcontrol_sse_open_event_t,
}

impl<'a> SseOpenEvent<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_sse_open_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'a> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return the response content type that identified this stream as SSE.
    pub fn content_type(&self) -> &[u8] {
        bytes_from_ptr_len(self.inner.content_type, self.inner.content_type_len)
    }
}

/// One parsed SSE event record.
pub struct SseEventRecord<'a> {
    inner: &'a mut ffi::qcontrol_sse_event_record_t,
}

impl<'a> SseEventRecord<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_sse_event_record_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'_> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return the parsed `event:` field, or an empty slice when absent.
    pub fn event_name(&self) -> &[u8] {
        event_bytes(self.inner, |ops| ops.event_name, |ops| ops.event_name_len)
            .unwrap_or_else(|| bytes_from_ptr_len(self.inner.event_name, self.inner.event_name_len))
    }

    /// Return the parsed `event:` field as UTF-8 when present.
    pub fn event_name_str(&self) -> Option<&str> {
        str::from_utf8(self.event_name()).ok()
    }

    /// Return the parsed `id:` field, or an empty slice when absent.
    pub fn id(&self) -> &[u8] {
        event_bytes(self.inner, |ops| ops.id, |ops| ops.id_len)
            .unwrap_or_else(|| bytes_from_ptr_len(self.inner.id, self.inner.id_len))
    }

    /// Return the parsed `id:` field as UTF-8 when present.
    pub fn id_str(&self) -> Option<&str> {
        str::from_utf8(self.id()).ok()
    }

    /// Return the parsed `retry:` field when present.
    pub fn retry_ms(&self) -> Option<u64> {
        if let Some(ops) = event_ops(self.inner) {
            if let Some(retry_fn) = ops.retry_ms {
                let mut retry_ms = 0u64;
                if unsafe { retry_fn(event_context(self.inner), &mut retry_ms) } == 0 {
                    return Some(retry_ms);
                }
            }
        }
        if self.inner.has_retry_ms != 0 {
            Some(self.inner.retry_ms)
        } else {
            None
        }
    }

    /// Return the parsed logical event data.
    pub fn data(&self) -> &[u8] {
        event_bytes(self.inner, |ops| ops.data, |ops| ops.data_len)
            .unwrap_or_else(|| bytes_from_ptr_len(self.inner.data, self.inner.data_len))
    }

    /// Return the parsed event data as UTF-8 when valid.
    pub fn data_str(&self) -> Option<&str> {
        str::from_utf8(self.data()).ok()
    }

    /// Return whether this SSE event supports in-place mutation.
    pub fn can_mutate(&self) -> bool {
        event_ops(self.inner).is_some()
    }

    /// Replace the parsed `event:` field.
    pub fn set_event_name(&mut self, value: &[u8]) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(set_fn) = ops.set_event_name else {
            return false;
        };
        unsafe {
            set_fn(
                event_context(self.inner),
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the parsed `event:` field with UTF-8 text.
    pub fn set_event_name_str(&mut self, value: &str) -> bool {
        self.set_event_name(value.as_bytes())
    }

    /// Clear the parsed `event:` field.
    pub fn clear_event_name(&mut self) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(clear_fn) = ops.clear_event_name else {
            return false;
        };
        unsafe { clear_fn(event_context(self.inner)) == 0 }
    }

    /// Replace the parsed `id:` field.
    pub fn set_id(&mut self, value: &[u8]) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(set_fn) = ops.set_id else {
            return false;
        };
        unsafe {
            set_fn(
                event_context(self.inner),
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the parsed `id:` field with UTF-8 text.
    pub fn set_id_str(&mut self, value: &str) -> bool {
        self.set_id(value.as_bytes())
    }

    /// Clear the parsed `id:` field.
    pub fn clear_id(&mut self) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(clear_fn) = ops.clear_id else {
            return false;
        };
        unsafe { clear_fn(event_context(self.inner)) == 0 }
    }

    /// Replace the parsed `retry:` field.
    pub fn set_retry_ms(&mut self, value: u64) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(set_fn) = ops.set_retry_ms else {
            return false;
        };
        unsafe { set_fn(event_context(self.inner), value) == 0 }
    }

    /// Clear the parsed `retry:` field.
    pub fn clear_retry_ms(&mut self) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(clear_fn) = ops.clear_retry_ms else {
            return false;
        };
        unsafe { clear_fn(event_context(self.inner)) == 0 }
    }

    /// Replace the parsed logical event data.
    pub fn set_data(&mut self, value: &[u8]) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(set_fn) = ops.set_data else {
            return false;
        };
        unsafe {
            set_fn(
                event_context(self.inner),
                value.as_ptr() as *const c_char,
                value.len(),
            ) == 0
        }
    }

    /// Replace the parsed logical event data with UTF-8 text.
    pub fn set_data_str(&mut self, value: &str) -> bool {
        self.set_data(value.as_bytes())
    }

    /// Clear the parsed logical event data.
    pub fn clear_data(&mut self) -> bool {
        let Some(ops) = event_ops(self.inner) else {
            return false;
        };
        let Some(clear_fn) = ops.clear_data else {
            return false;
        };
        unsafe { clear_fn(event_context(self.inner)) == 0 }
    }
}

/// Event delivered when the owning SSE response stream closes.
pub struct SseCloseEvent<'a> {
    inner: &'a ffi::qcontrol_sse_close_event_t,
}

impl<'a> SseCloseEvent<'a> {
    /// Create from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_sse_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Return the shared stream correlation context.
    pub fn ctx(&self) -> StreamContext<'a> {
        unsafe { StreamContext::from_ref(&self.inner.ctx) }
    }

    /// Return why the owning HTTP response stream closed.
    pub fn reason(&self) -> HttpCloseReason {
        HttpCloseReason::from_ffi(self.inner.reason)
    }
}
