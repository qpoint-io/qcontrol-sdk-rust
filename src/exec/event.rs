//! Exec event types
//!
//! Lifetime-bound wrappers around C event structures.

use std::ffi::CStr;

use crate::ffi;

/// Event for exec operations (execve, posix_spawn, etc.).
///
/// Provides access to the path, arguments, environment, and cwd.
pub struct ExecEvent<'a> {
    inner: &'a ffi::qcontrol_exec_event_t,
}

impl<'a> ExecEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the executable path.
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

    /// Get the executable path as bytes.
    pub fn path_bytes(&self) -> &[u8] {
        if self.inner.path.is_null() || self.inner.path_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.inner.path as *const u8, self.inner.path_len) }
        }
    }

    /// Get the number of arguments.
    pub fn argc(&self) -> usize {
        self.inner.argc
    }

    /// Iterate over the arguments.
    pub fn argv(&self) -> impl Iterator<Item = &str> + '_ {
        ArgvIterator {
            argv: self.inner.argv,
            count: self.inner.argc,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the number of environment variables.
    pub fn envc(&self) -> usize {
        self.inner.envc
    }

    /// Iterate over the environment as KEY=VALUE pairs.
    pub fn envp(&self) -> impl Iterator<Item = &str> + '_ {
        ArgvIterator {
            argv: self.inner.envp,
            count: self.inner.envc,
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the working directory (may be None if not specified).
    pub fn cwd(&self) -> Option<&str> {
        if self.inner.cwd.is_null() || self.inner.cwd_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.cwd).to_str().ok() }
        }
    }
}

/// Event for stdin data being written to child process.
pub struct StdinEvent<'a> {
    inner: &'a ffi::qcontrol_exec_stdin_event_t,
}

impl<'a> StdinEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_stdin_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the child process ID.
    pub fn pid(&self) -> i32 {
        self.inner.pid
    }

    /// Get the number of bytes being written.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the data being written.
    pub fn data(&self) -> &[u8] {
        if self.inner.buf.is_null() || self.inner.count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.inner.buf as *const u8, self.inner.count) }
        }
    }

    /// Get the data as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        std::str::from_utf8(self.data()).ok()
    }
}

/// Event for stdout data read from child process.
pub struct StdoutEvent<'a> {
    inner: &'a ffi::qcontrol_exec_stdout_event_t,
}

impl<'a> StdoutEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_stdout_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the child process ID.
    pub fn pid(&self) -> i32 {
        self.inner.pid
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

    /// Get the data read (only valid after successful read).
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

    /// Get the data as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        self.data().and_then(|d| std::str::from_utf8(d).ok())
    }
}

/// Event for stderr data read from child process.
pub struct StderrEvent<'a> {
    inner: &'a ffi::qcontrol_exec_stderr_event_t,
}

impl<'a> StderrEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_stderr_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the child process ID.
    pub fn pid(&self) -> i32 {
        self.inner.pid
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

    /// Get the data read (only valid after successful read).
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

    /// Get the data as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        self.data().and_then(|d| std::str::from_utf8(d).ok())
    }
}

/// Event for child process exit.
pub struct ExitEvent<'a> {
    inner: &'a ffi::qcontrol_exec_exit_event_t,
}

impl<'a> ExitEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_exit_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the child process ID.
    pub fn pid(&self) -> i32 {
        self.inner.pid
    }

    /// Get the exit code (only valid if exit_signal is 0).
    pub fn exit_code(&self) -> i32 {
        self.inner.exit_code
    }

    /// Get the signal number that killed the process (0 if normal exit).
    pub fn exit_signal(&self) -> i32 {
        self.inner.exit_signal
    }

    /// Check if the process exited normally (not killed by signal).
    pub fn exited_normally(&self) -> bool {
        self.inner.exit_signal == 0
    }
}

/// Iterator over argv/envp arrays.
struct ArgvIterator<'a> {
    argv: *const *const std::ffi::c_char,
    count: usize,
    index: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Iterator for ArgvIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.argv.is_null() || self.index >= self.count {
            return None;
        }
        unsafe {
            let ptr = *self.argv.add(self.index);
            self.index += 1;
            if ptr.is_null() {
                None
            } else {
                CStr::from_ptr(ptr).to_str().ok()
            }
        }
    }
}
