//! File operation context types

use std::ffi::CStr;
use std::path::Path;
use std::ptr;

use crate::ffi::{RawFileCloseCtx, RawFileOpenCtx, RawFileReadCtx, RawFileWriteCtx, MAX_PATH};

/// Context for open() operations.
pub struct FileOpenContext<'a> {
    pub(crate) inner: &'a mut RawFileOpenCtx,
}

impl<'a> FileOpenContext<'a> {
    /// Get the file path being opened.
    pub fn path(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.inner.path)
                .to_str()
                .unwrap_or("<invalid utf8>")
        }
    }

    /// Get the file path as a Path.
    pub fn path_as_path(&self) -> &Path {
        Path::new(self.path())
    }

    /// Get the open flags.
    pub fn flags(&self) -> i32 {
        self.inner.flags
    }

    /// Get the file mode (for O_CREAT).
    pub fn mode(&self) -> u32 {
        self.inner.mode
    }

    /// Get the result (fd on success, negative errno on error).
    /// Only meaningful in Leave phase.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the open succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result >= 0
    }

    /// Set a modified path for the operation.
    /// Only effective in Enter phase with FilterResult::Modify.
    pub fn set_path(&mut self, new_path: &str) {
        let bytes = new_path.as_bytes();
        let len = bytes.len().min(MAX_PATH - 1);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.inner.path_out, len);
            *self.inner.path_out.add(len) = 0;
        }
    }
}

/// Context for read() operations.
pub struct FileReadContext<'a> {
    pub(crate) inner: &'a mut RawFileReadCtx,
}

impl<'a> FileReadContext<'a> {
    /// Get the file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes requested.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the result (bytes read, or negative errno).
    /// Only meaningful in Leave phase.
    pub fn result(&self) -> isize {
        self.inner.result
    }

    /// Get the buffer contents (only valid in Leave phase after successful read).
    pub fn buffer(&self) -> Option<&[u8]> {
        if self.inner.result > 0 {
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
}

/// Context for write() operations.
pub struct FileWriteContext<'a> {
    pub(crate) inner: &'a mut RawFileWriteCtx,
}

impl<'a> FileWriteContext<'a> {
    /// Get the file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes to write.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the result (bytes written, or negative errno).
    /// Only meaningful in Leave phase.
    pub fn result(&self) -> isize {
        self.inner.result
    }

    /// Get the buffer being written.
    pub fn buffer(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.inner.buf as *const u8, self.inner.count) }
    }
}

/// Context for close() operations.
pub struct FileCloseContext<'a> {
    pub(crate) inner: &'a mut RawFileCloseCtx,
}

impl<'a> FileCloseContext<'a> {
    /// Get the file descriptor being closed.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the result (0 on success, negative errno on error).
    /// Only meaningful in Leave phase.
    pub fn result(&self) -> i32 {
        self.inner.result
    }
}
