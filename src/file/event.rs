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
            unsafe { std::slice::from_raw_parts(self.inner.path as *const u8, self.inner.path_len) }
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
            unsafe { std::slice::from_raw_parts(self.inner.buf as *const u8, self.inner.count) }
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

/// Coarse OpenShell-style read-versus-write bucket for file path operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePolicyClass {
    /// Operation only reads path metadata or directory structure.
    Read,
    /// Operation mutates path state or permissions.
    Write,
}

impl From<ffi::qcontrol_file_policy_class_t> for FilePolicyClass {
    fn from(value: ffi::qcontrol_file_policy_class_t) -> Self {
        match value {
            ffi::qcontrol_file_policy_class_t_QCONTROL_FILE_POLICY_CLASS_WRITE => Self::Write,
            _ => Self::Read,
        }
    }
}

/// Path-oriented filesystem operation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOpKind {
    Access,
    Stat,
    Readlink,
    Opendir,
    Rename,
    Unlink,
    Mkdir,
    Rmdir,
    Link,
    Symlink,
    Chmod,
    Chown,
}

impl From<ffi::qcontrol_file_op_kind_t> for FileOpKind {
    fn from(value: ffi::qcontrol_file_op_kind_t) -> Self {
        match value {
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_STAT => Self::Stat,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_READLINK => Self::Readlink,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_OPENDIR => Self::Opendir,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_RENAME => Self::Rename,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_UNLINK => Self::Unlink,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_MKDIR => Self::Mkdir,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_RMDIR => Self::Rmdir,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_LINK => Self::Link,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_SYMLINK => Self::Symlink,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_CHMOD => Self::Chmod,
            ffi::qcontrol_file_op_kind_t_QCONTROL_FILE_OP_CHOWN => Self::Chown,
            _ => Self::Access,
        }
    }
}

/// Event for path-oriented filesystem operations outside the fd stream model.
pub struct FileOpEvent<'a> {
    inner: &'a ffi::qcontrol_file_op_event_t,
}

impl<'a> FileOpEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_op_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the filesystem operation kind.
    pub fn kind(&self) -> FileOpKind {
        self.inner.kind.into()
    }

    /// Get the coarse OpenShell-style read/write policy class.
    pub fn policy_class(&self) -> FilePolicyClass {
        self.inner.policy_class.into()
    }

    /// Get the primary path argument.
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

    /// Get the secondary path argument when the operation uses two paths.
    pub fn secondary_path(&self) -> Option<&str> {
        if self.inner.secondary_path.is_null() || self.inner.secondary_path_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.secondary_path).to_str().ok() }
        }
    }

    /// Get the dirfd context for `*at`-style operations.
    pub fn dirfd(&self) -> i32 {
        self.inner.dirfd
    }

    /// Get the secondary dirfd context for two-path `*at`-style operations.
    pub fn secondary_dirfd(&self) -> i32 {
        self.inner.secondary_dirfd
    }

    /// Get the operation flags such as `AT_*`.
    pub fn at_flags(&self) -> i32 {
        self.inner.at_flags
    }

    /// Get the raw operation result.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check whether the operation succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result == 0
    }
}
