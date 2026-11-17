//! Buffer wrapper for transform operations.
//!
//! This module wraps one host-backed buffer handle whose embedded callback
//! table exposes the shared mutation surface across file, net, exec, and HTTP
//! plugins.

use std::ffi::{c_char, c_void};

use crate::ffi;

/// Returns the callback table attached to one raw buffer handle.
fn buffer_ops(raw: *const ffi::qcontrol_buffer_t) -> Option<&'static ffi::qcontrol_buffer_ops_t> {
    unsafe { raw.as_ref()?.ops.as_ref() }
}

/// Returns the host-owned context pointer attached to one raw buffer handle.
fn buffer_context(raw: *const ffi::qcontrol_buffer_t) -> *mut c_void {
    unsafe { raw.as_ref().map_or(std::ptr::null_mut(), |buf| buf.context) }
}

/// Returns the current length reported by one raw buffer handle.
fn buffer_len(raw: *const ffi::qcontrol_buffer_t) -> usize {
    let Some(ops) = buffer_ops(raw) else {
        return 0;
    };
    let Some(len_fn) = ops.len else {
        return 0;
    };
    unsafe { len_fn(buffer_context(raw)) }
}

/// Returns the current data pointer reported by one raw buffer handle.
fn buffer_data(raw: *const ffi::qcontrol_buffer_t) -> *const c_char {
    let Some(ops) = buffer_ops(raw) else {
        return std::ptr::null();
    };
    let Some(data_fn) = ops.data else {
        return std::ptr::null();
    };
    unsafe { data_fn(buffer_context(raw)) }
}

/// Returns the current byte slice exposed by one raw buffer handle.
pub(crate) fn buffer_bytes(raw: *const ffi::qcontrol_buffer_t) -> &'static [u8] {
    let len = buffer_len(raw);
    if len == 0 {
        return &[];
    }

    let ptr = buffer_data(raw);
    if ptr.is_null() {
        return &[];
    }

    unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
}

/// Reports whether one raw buffer handle contains the requested bytes.
fn buffer_contains(raw: *const ffi::qcontrol_buffer_t, needle: &[u8]) -> bool {
    let Some(ops) = buffer_ops(raw) else {
        return false;
    };
    let Some(contains_fn) = ops.contains else {
        return false;
    };
    unsafe {
        contains_fn(
            buffer_context(raw),
            needle.as_ptr() as *const c_char,
            needle.len(),
        ) != 0
    }
}

/// Reports whether one raw buffer handle starts with the requested prefix.
fn buffer_starts_with(raw: *const ffi::qcontrol_buffer_t, prefix: &[u8]) -> bool {
    let Some(ops) = buffer_ops(raw) else {
        return false;
    };
    let Some(starts_with_fn) = ops.starts_with else {
        return false;
    };
    unsafe {
        starts_with_fn(
            buffer_context(raw),
            prefix.as_ptr() as *const c_char,
            prefix.len(),
        ) != 0
    }
}

/// Reports whether one raw buffer handle ends with the requested suffix.
fn buffer_ends_with(raw: *const ffi::qcontrol_buffer_t, suffix: &[u8]) -> bool {
    let Some(ops) = buffer_ops(raw) else {
        return false;
    };
    let Some(ends_with_fn) = ops.ends_with else {
        return false;
    };
    unsafe {
        ends_with_fn(
            buffer_context(raw),
            suffix.as_ptr() as *const c_char,
            suffix.len(),
        ) != 0
    }
}

/// Finds the first index of one byte sequence within the raw buffer handle.
fn buffer_find(raw: *const ffi::qcontrol_buffer_t, needle: &[u8]) -> Option<usize> {
    let ops = buffer_ops(raw)?;
    let index_of_fn = ops.index_of?;
    let index = unsafe {
        index_of_fn(
            buffer_context(raw),
            needle.as_ptr() as *const c_char,
            needle.len(),
        )
    };
    if index == usize::MAX {
        None
    } else {
        Some(index)
    }
}

/// Invokes one void mutation callback on the raw buffer handle when present.
fn buffer_write(
    raw: *mut ffi::qcontrol_buffer_t,
    f: impl FnOnce(&ffi::qcontrol_buffer_ops_t, *mut c_void),
) {
    let Some(ops) = buffer_ops(raw) else {
        return;
    };
    f(ops, buffer_context(raw));
}

/// A read-only view over one host-managed buffer.
///
/// This type mirrors the query surface of [`Buffer`] for APIs that expose a
/// paired `foo()` / `foo_mut()` access pattern.
pub struct BufferRef<'a> {
    inner: *mut ffi::qcontrol_buffer_t,
    _marker: std::marker::PhantomData<&'a ffi::qcontrol_buffer_t>,
}

impl<'a> BufferRef<'a> {
    /// Create a new read-only buffer wrapper from a raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_buffer_t) -> Self {
        Self {
            inner: ptr,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the current length of the buffer in bytes.
    pub fn len(&self) -> usize {
        buffer_len(self.inner)
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer contents as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        buffer_bytes(self.inner)
    }

    /// Get the buffer contents as a string slice if valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_slice()).ok()
    }

    /// Check if the buffer contains the given needle.
    pub fn contains(&self, needle: &[u8]) -> bool {
        buffer_contains(self.inner, needle)
    }

    /// Check if the buffer contains the given string.
    pub fn contains_str(&self, needle: &str) -> bool {
        self.contains(needle.as_bytes())
    }

    /// Check if the buffer starts with the given prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        buffer_starts_with(self.inner, prefix)
    }

    /// Check if the buffer starts with the given string.
    pub fn starts_with_str(&self, prefix: &str) -> bool {
        self.starts_with(prefix.as_bytes())
    }

    /// Check if the buffer ends with the given suffix.
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        buffer_ends_with(self.inner, suffix)
    }

    /// Check if the buffer ends with the given string.
    pub fn ends_with_str(&self, suffix: &str) -> bool {
        self.ends_with(suffix.as_bytes())
    }

    /// Find the index of the first occurrence of one byte sequence.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        buffer_find(self.inner, needle)
    }

    /// Find the index of the first occurrence of one UTF-8 string.
    pub fn find_str(&self, needle: &str) -> Option<usize> {
        self.find(needle.as_bytes())
    }
}

/// A mutable buffer for transform operations.
///
/// This type wraps one host-managed buffer handle and exposes the repository's
/// shared query and mutation helpers through that handle's callback table.
pub struct Buffer<'a> {
    inner: &'a mut ffi::qcontrol_buffer_t,
}

impl<'a> Buffer<'a> {
    /// Create a new buffer wrapper from one raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_buffer_t) -> Self {
        Self { inner: &mut *ptr }
    }

    /// Get the current length of the buffer in bytes.
    pub fn len(&self) -> usize {
        buffer_len(self.inner)
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer contents as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        buffer_bytes(self.inner)
    }

    /// Get the buffer contents as a string slice if valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_slice()).ok()
    }

    /// Check if the buffer contains the given needle.
    pub fn contains(&self, needle: &[u8]) -> bool {
        buffer_contains(self.inner, needle)
    }

    /// Check if the buffer contains the given string.
    pub fn contains_str(&self, needle: &str) -> bool {
        self.contains(needle.as_bytes())
    }

    /// Check if the buffer starts with the given prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        buffer_starts_with(self.inner, prefix)
    }

    /// Check if the buffer starts with the given string.
    pub fn starts_with_str(&self, prefix: &str) -> bool {
        self.starts_with(prefix.as_bytes())
    }

    /// Check if the buffer ends with the given suffix.
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        buffer_ends_with(self.inner, suffix)
    }

    /// Check if the buffer ends with the given string.
    pub fn ends_with_str(&self, suffix: &str) -> bool {
        self.ends_with(suffix.as_bytes())
    }

    /// Find the index of the first occurrence of needle.
    /// Returns `None` if not found.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        buffer_find(self.inner, needle)
    }

    /// Find the index of the first occurrence of the string.
    /// Returns `None` if not found.
    pub fn find_str(&self, needle: &str) -> Option<usize> {
        self.find(needle.as_bytes())
    }

    /// Prepend data to the beginning of the buffer.
    pub fn prepend(&mut self, data: &[u8]) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(prepend_fn) = ops.prepend {
                unsafe { prepend_fn(ctx, data.as_ptr() as *const c_char, data.len()) };
            }
        });
    }

    /// Prepend a string to the beginning of the buffer.
    pub fn prepend_str(&mut self, data: &str) {
        self.prepend(data.as_bytes())
    }

    /// Append data to the end of the buffer.
    pub fn append(&mut self, data: &[u8]) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(append_fn) = ops.append {
                unsafe { append_fn(ctx, data.as_ptr() as *const c_char, data.len()) };
            }
        });
    }

    /// Append a string to the end of the buffer.
    pub fn append_str(&mut self, data: &str) {
        self.append(data.as_bytes())
    }

    /// Replace the first occurrence of needle with replacement.
    /// Returns `true` if a replacement was made.
    pub fn replace(&mut self, needle: &[u8], replacement: &[u8]) -> bool {
        let Some(ops) = buffer_ops(self.inner) else {
            return false;
        };
        let Some(replace_fn) = ops.replace else {
            return false;
        };
        unsafe {
            replace_fn(
                buffer_context(self.inner),
                needle.as_ptr() as *const c_char,
                needle.len(),
                replacement.as_ptr() as *const c_char,
                replacement.len(),
            ) != 0
        }
    }

    /// Replace the first occurrence of needle string with replacement.
    /// Returns `true` if a replacement was made.
    pub fn replace_str(&mut self, needle: &str, replacement: &str) -> bool {
        self.replace(needle.as_bytes(), replacement.as_bytes())
    }

    /// Replace all occurrences of needle with replacement.
    /// Returns the number of replacements made.
    pub fn replace_all(&mut self, needle: &[u8], replacement: &[u8]) -> usize {
        let Some(ops) = buffer_ops(self.inner) else {
            return 0;
        };
        let Some(replace_all_fn) = ops.replace_all else {
            return 0;
        };
        unsafe {
            replace_all_fn(
                buffer_context(self.inner),
                needle.as_ptr() as *const c_char,
                needle.len(),
                replacement.as_ptr() as *const c_char,
                replacement.len(),
            )
        }
    }

    /// Replace all occurrences of needle string with replacement.
    /// Returns the number of replacements made.
    pub fn replace_all_str(&mut self, needle: &str, replacement: &str) -> usize {
        self.replace_all(needle.as_bytes(), replacement.as_bytes())
    }

    /// Remove the first occurrence of needle.
    /// Returns `true` if a removal was made.
    pub fn remove(&mut self, needle: &[u8]) -> bool {
        let Some(ops) = buffer_ops(self.inner) else {
            return false;
        };
        let Some(remove_fn) = ops.remove else {
            return false;
        };
        unsafe {
            remove_fn(
                buffer_context(self.inner),
                needle.as_ptr() as *const c_char,
                needle.len(),
            ) != 0
        }
    }

    /// Remove the first occurrence of needle string.
    /// Returns `true` if a removal was made.
    pub fn remove_str(&mut self, needle: &str) -> bool {
        self.remove(needle.as_bytes())
    }

    /// Remove all occurrences of needle.
    /// Returns the number of removals made.
    pub fn remove_all(&mut self, needle: &[u8]) -> usize {
        let Some(ops) = buffer_ops(self.inner) else {
            return 0;
        };
        let Some(remove_all_fn) = ops.remove_all else {
            return 0;
        };
        unsafe {
            remove_all_fn(
                buffer_context(self.inner),
                needle.as_ptr() as *const c_char,
                needle.len(),
            )
        }
    }

    /// Remove all occurrences of needle string.
    /// Returns the number of removals made.
    pub fn remove_all_str(&mut self, needle: &str) -> usize {
        self.remove_all(needle.as_bytes())
    }

    /// Clear the buffer contents.
    pub fn clear(&mut self) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(clear_fn) = ops.clear {
                unsafe { clear_fn(ctx) };
            }
        });
    }

    /// Set the buffer contents to new data.
    pub fn set(&mut self, data: &[u8]) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(set_fn) = ops.set {
                unsafe { set_fn(ctx, data.as_ptr() as *const c_char, data.len()) };
            }
        });
    }

    /// Set the buffer contents to a string.
    pub fn set_str(&mut self, data: &str) {
        self.set(data.as_bytes())
    }

    /// Insert data at the given position.
    pub fn insert_at(&mut self, pos: usize, data: &[u8]) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(insert_at_fn) = ops.insert_at {
                unsafe { insert_at_fn(ctx, pos, data.as_ptr() as *const c_char, data.len()) };
            }
        });
    }

    /// Insert a string at the given position.
    pub fn insert_at_str(&mut self, pos: usize, data: &str) {
        self.insert_at(pos, data.as_bytes())
    }

    /// Remove a range of bytes from the buffer.
    pub fn remove_range(&mut self, start: usize, end: usize) {
        buffer_write(self.inner, |ops, ctx| {
            if let Some(remove_range_fn) = ops.remove_range {
                unsafe { remove_range_fn(ctx, start, end) };
            }
        });
    }
}
