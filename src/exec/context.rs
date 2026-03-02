//! Exec context for transform functions
//!
//! Provides metadata about the process being operated on.

use std::ffi::CStr;

use crate::ffi;

/// Exec context passed to transform functions.
///
/// Provides metadata about the child process.
pub struct ExecContext<'a> {
    inner: &'a ffi::qcontrol_exec_ctx_t,
}

impl<'a> ExecContext<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_exec_ctx_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the child process ID.
    pub fn pid(&self) -> i32 {
        self.inner.pid
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
}

/// Iterator over argv arrays.
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
