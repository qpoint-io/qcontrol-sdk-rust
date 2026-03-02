//! File event types
//!
//! Lifetime-bound wrappers around C event structures.

use std::ffi::CStr;
use std::path::Path;

use crate::ffi;

/// Event for file open operations.
///
/// Provides access to the path, flags, mode, and result of an open() call.
pub struct FileOpenEvent<'a> {
    inner: &'a ffi::qcontrol_file_open_event_t,
}

impl<'a> FileOpenEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_open_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the file path being opened.
    pub fn path(&self) -> &str {
        if self.inner.path.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.path)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }

    /// Get the file path as bytes.
    pub fn path_bytes(&self) -> &[u8] {
        if self.inner.path.is_null() || self.inner.path_len == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(self.inner.path as *const u8, self.inner.path_len)
            }
        }
    }

    /// Get the file path as a Path.
    pub fn path_as_path(&self) -> &Path {
        Path::new(self.path())
    }

    /// Get the open flags (O_RDONLY, O_WRONLY, etc.).
    pub fn flags(&self) -> i32 {
        self.inner.flags
    }

    /// Get the file mode (for O_CREAT).
    pub fn mode(&self) -> u32 {
        self.inner.mode
    }

    /// Get the result of the open operation.
    ///
    /// On success, returns the file descriptor (>= 0).
    /// On failure, returns a negative errno value.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the open succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result >= 0
    }

    /// Get the file descriptor if the open succeeded.
    pub fn fd(&self) -> Option<i32> {
        if self.succeeded() {
            Some(self.inner.result)
        } else {
            None
        }
    }
}

/// Event for file read operations.
///
/// Provides access to the fd, buffer, and result of a read() call.
pub struct FileReadEvent<'a> {
    inner: &'a ffi::qcontrol_file_read_event_t,
}

impl<'a> FileReadEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_read_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes requested.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the result of the read operation.
    ///
    /// On success, returns the number of bytes read (>= 0).
    /// On failure, returns a negative errno value.
    pub fn result(&self) -> isize {
        self.inner.result
    }

    /// Get the buffer contents (only valid after successful read).
    pub fn data(&self) -> Option<&[u8]> {
        if self.inner.result > 0 && !self.inner.buf.is_null() {
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.inner.buf as *const u8,
                    self.inner.result as usize,
                ))
            }
        } else {
            None
        }
    }

    /// Get the buffer contents as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        self.data().and_then(|d| std::str::from_utf8(d).ok())
    }
}

/// Event for file write operations.
///
/// Provides access to the fd, buffer, and result of a write() call.
pub struct FileWriteEvent<'a> {
    inner: &'a ffi::qcontrol_file_write_event_t,
}

impl<'a> FileWriteEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_write_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes to write.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the result of the write operation.
    ///
    /// On success, returns the number of bytes written (>= 0).
    /// On failure, returns a negative errno value.
    pub fn result(&self) -> isize {
        self.inner.result
    }

    /// Get the buffer being written.
    pub fn data(&self) -> &[u8] {
        if self.inner.buf.is_null() || self.inner.count == 0 {
            &[]
        } else {
            unsafe {
                std::slice::from_raw_parts(self.inner.buf as *const u8, self.inner.count)
            }
        }
    }

    /// Get the buffer contents as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        std::str::from_utf8(self.data()).ok()
    }
}

/// Event for file close operations.
///
/// Provides access to the fd and result of a close() call.
pub struct FileCloseEvent<'a> {
    inner: &'a ffi::qcontrol_file_close_event_t,
}

impl<'a> FileCloseEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the file descriptor being closed.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the result of the close operation.
    ///
    /// On success, returns 0.
    /// On failure, returns a negative errno value.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the close succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result == 0
    }
}
