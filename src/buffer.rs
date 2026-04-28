//! Buffer wrapper for transform operations
//!
//! Thin wrapper around the agent's C buffer functions. These functions
//! are implemented by the agent and called by plugins during transforms.

use std::ffi::c_char;

use crate::ffi;

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
        unsafe { ffi::qcontrol_buffer_len(self.inner) }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer contents as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let ptr = ffi::qcontrol_buffer_data(self.inner);
            let len = ffi::qcontrol_buffer_len(self.inner);
            if ptr.is_null() || len == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(ptr as *const u8, len)
            }
        }
    }

    /// Get the buffer contents as a string slice if valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_slice()).ok()
    }

    /// Check if the buffer contains the given needle.
    pub fn contains(&self, needle: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_contains(
                self.inner,
                needle.as_ptr() as *const c_char,
                needle.len(),
            ) != 0
        }
    }

    /// Check if the buffer contains the given string.
    pub fn contains_str(&self, needle: &str) -> bool {
        self.contains(needle.as_bytes())
    }

    /// Check if the buffer starts with the given prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_starts_with(
                self.inner,
                prefix.as_ptr() as *const c_char,
                prefix.len(),
            ) != 0
        }
    }

    /// Check if the buffer starts with the given string.
    pub fn starts_with_str(&self, prefix: &str) -> bool {
        self.starts_with(prefix.as_bytes())
    }

    /// Check if the buffer ends with the given suffix.
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_ends_with(
                self.inner,
                suffix.as_ptr() as *const c_char,
                suffix.len(),
            ) != 0
        }
    }

    /// Check if the buffer ends with the given string.
    pub fn ends_with_str(&self, suffix: &str) -> bool {
        self.ends_with(suffix.as_bytes())
    }

    /// Find the index of the first occurrence of one byte sequence.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        let index = unsafe {
            ffi::qcontrol_buffer_index_of(
                self.inner,
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

    /// Find the index of the first occurrence of one UTF-8 string.
    pub fn find_str(&self, needle: &str) -> Option<usize> {
        self.find(needle.as_bytes())
    }
}

/// A mutable buffer for transform operations.
///
/// This type wraps the agent's buffer implementation and provides
/// methods for reading and modifying buffer contents during transforms.
pub struct Buffer<'a> {
    inner: &'a mut ffi::qcontrol_buffer_t,
}

impl<'a> Buffer<'a> {
    /// Create a new Buffer wrapper from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_buffer_t) -> Self {
        Self { inner: &mut *ptr }
    }

    // ========================================================================
    // Read operations
    // ========================================================================

    /// Get the current length of the buffer in bytes.
    pub fn len(&self) -> usize {
        unsafe { ffi::qcontrol_buffer_len(self.inner) }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the buffer contents as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let ptr = ffi::qcontrol_buffer_data(self.inner);
            let len = ffi::qcontrol_buffer_len(self.inner);
            if ptr.is_null() || len == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(ptr as *const u8, len)
            }
        }
    }

    /// Get the buffer contents as a string slice if valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.as_slice()).ok()
    }

    /// Check if the buffer contains the given needle.
    pub fn contains(&self, needle: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_contains(
                self.inner,
                needle.as_ptr() as *const c_char,
                needle.len(),
            ) != 0
        }
    }

    /// Check if the buffer contains the given string.
    pub fn contains_str(&self, needle: &str) -> bool {
        self.contains(needle.as_bytes())
    }

    /// Check if the buffer starts with the given prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_starts_with(
                self.inner,
                prefix.as_ptr() as *const c_char,
                prefix.len(),
            ) != 0
        }
    }

    /// Check if the buffer starts with the given string.
    pub fn starts_with_str(&self, prefix: &str) -> bool {
        self.starts_with(prefix.as_bytes())
    }

    /// Check if the buffer ends with the given suffix.
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_ends_with(
                self.inner,
                suffix.as_ptr() as *const c_char,
                suffix.len(),
            ) != 0
        }
    }

    /// Check if the buffer ends with the given string.
    pub fn ends_with_str(&self, suffix: &str) -> bool {
        self.ends_with(suffix.as_bytes())
    }

    /// Find the index of the first occurrence of needle.
    /// Returns `None` if not found.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        let index = unsafe {
            ffi::qcontrol_buffer_index_of(
                self.inner,
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

    /// Find the index of the first occurrence of the string.
    /// Returns `None` if not found.
    pub fn find_str(&self, needle: &str) -> Option<usize> {
        self.find(needle.as_bytes())
    }

    // ========================================================================
    // Write operations
    // ========================================================================

    /// Prepend data to the beginning of the buffer.
    pub fn prepend(&mut self, data: &[u8]) {
        unsafe {
            ffi::qcontrol_buffer_prepend(self.inner, data.as_ptr() as *const c_char, data.len());
        }
    }

    /// Prepend a string to the beginning of the buffer.
    pub fn prepend_str(&mut self, data: &str) {
        self.prepend(data.as_bytes())
    }

    /// Append data to the end of the buffer.
    pub fn append(&mut self, data: &[u8]) {
        unsafe {
            ffi::qcontrol_buffer_append(self.inner, data.as_ptr() as *const c_char, data.len());
        }
    }

    /// Append a string to the end of the buffer.
    pub fn append_str(&mut self, data: &str) {
        self.append(data.as_bytes())
    }

    /// Replace the first occurrence of needle with replacement.
    /// Returns `true` if a replacement was made.
    pub fn replace(&mut self, needle: &[u8], replacement: &[u8]) -> bool {
        unsafe {
            ffi::qcontrol_buffer_replace(
                self.inner,
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
        unsafe {
            ffi::qcontrol_buffer_replace_all(
                self.inner,
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
        unsafe {
            ffi::qcontrol_buffer_remove(self.inner, needle.as_ptr() as *const c_char, needle.len())
                != 0
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
        unsafe {
            ffi::qcontrol_buffer_remove_all(
                self.inner,
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
        unsafe {
            ffi::qcontrol_buffer_clear(self.inner);
        }
    }

    /// Set the buffer contents to new data.
    pub fn set(&mut self, data: &[u8]) {
        unsafe {
            ffi::qcontrol_buffer_set(self.inner, data.as_ptr() as *const c_char, data.len());
        }
    }

    /// Set the buffer contents to a string.
    pub fn set_str(&mut self, data: &str) {
        self.set(data.as_bytes())
    }

    /// Insert data at the given position.
    pub fn insert_at(&mut self, pos: usize, data: &[u8]) {
        unsafe {
            ffi::qcontrol_buffer_insert_at(
                self.inner,
                pos,
                data.as_ptr() as *const c_char,
                data.len(),
            );
        }
    }

    /// Insert a string at the given position.
    pub fn insert_at_str(&mut self, pos: usize, data: &str) {
        self.insert_at(pos, data.as_bytes())
    }

    /// Remove a range of bytes from the buffer.
    pub fn remove_range(&mut self, start: usize, end: usize) {
        unsafe {
            ffi::qcontrol_buffer_remove_range(self.inner, start, end);
        }
    }
}
