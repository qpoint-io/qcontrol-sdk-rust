//! qcontrol SDK for Rust
//!
//! This crate provides idiomatic Rust bindings for the qcontrol plugin SDK,
//! enabling you to write file operation filters in safe Rust.
//!
//! # Example
//!
//! ```rust,ignore
//! use qcontrol::{plugin, register_file_open, FileOpenContext, FilterResult, Error};
//!
//! fn on_open(ctx: &FileOpenContext) -> FilterResult {
//!     eprintln!("open({}) = {}", ctx.path(), ctx.result());
//!     FilterResult::Continue
//! }
//!
//! plugin!(|| -> Result<(), Error> {
//!     register_file_open("my_plugin", None, Some(on_open))?;
//!     Ok(())
//! });
//! ```

use std::ffi::{c_char, c_void, CStr};
use std::fmt;
use std::path::Path;
use std::ptr;

// Include bindgen-generated bindings from C headers
// This ensures C headers are the single source of truth for ABI types
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// ============================================================================
// Public API Types
// ============================================================================

/// Result of a filter callback, determining how the operation proceeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    /// Continue to the next filter in the chain
    Continue,
    /// Continue but apply any modifications made to the context
    Modify,
    /// Block the operation entirely (returns error to caller)
    Block,
}

/// Error codes from SDK operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An invalid argument was provided (e.g., name contains null bytes).
    InvalidArg,
    /// Memory allocation failed.
    NoMemory,
    /// The SDK is not initialized.
    NotInitialized,
    /// The specified plugin was not found.
    PluginNotFound,
    /// Plugin initialization failed.
    PluginInitFailed,
    /// Filter registration failed.
    RegisterFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidArg => write!(f, "invalid argument"),
            Error::NoMemory => write!(f, "memory allocation failed"),
            Error::NotInitialized => write!(f, "SDK not initialized"),
            Error::PluginNotFound => write!(f, "plugin not found"),
            Error::PluginInitFailed => write!(f, "plugin initialization failed"),
            Error::RegisterFailed => write!(f, "filter registration failed"),
        }
    }
}

impl std::error::Error for Error {}

/// Handle for a registered filter, used for unregistration.
#[derive(Debug, Clone, Copy)]
pub struct FilterHandle(ffi::qcontrol_filter_handle_t);

// ============================================================================
// Safe Context Wrappers
// ============================================================================

/// Context for open() operations.
pub struct FileOpenContext<'a> {
    inner: &'a mut RawFileOpenCtx,
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
    inner: &'a mut RawFileReadCtx,
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
    inner: &'a mut RawFileWriteCtx,
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
    inner: &'a mut RawFileCloseCtx,
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

// ============================================================================
// Filter Registration (Safe API)
// ============================================================================

/// Enter callback for open filter - receives mutable context for modifications.
pub type FileOpenEnterFn = fn(&mut FileOpenContext) -> FilterResult;
/// Leave callback for open filter - receives immutable context (read-only).
pub type FileOpenLeaveFn = fn(&FileOpenContext) -> FilterResult;

/// Enter callback for read filter - receives mutable context for modifications.
pub type FileReadEnterFn = fn(&mut FileReadContext) -> FilterResult;
/// Leave callback for read filter - receives immutable context (read-only).
pub type FileReadLeaveFn = fn(&FileReadContext) -> FilterResult;

/// Enter callback for write filter - receives mutable context for modifications.
pub type FileWriteEnterFn = fn(&mut FileWriteContext) -> FilterResult;
/// Leave callback for write filter - receives immutable context (read-only).
pub type FileWriteLeaveFn = fn(&FileWriteContext) -> FilterResult;

/// Enter callback for close filter - receives mutable context for modifications.
pub type FileCloseEnterFn = fn(&mut FileCloseContext) -> FilterResult;
/// Leave callback for close filter - receives immutable context (read-only).
pub type FileCloseLeaveFn = fn(&FileCloseContext) -> FilterResult;

/// Register an open filter using simple functions.
///
/// # Arguments
/// * `name` - A unique name for this filter
/// * `on_enter` - Callback invoked before the open() call (can modify path)
/// * `on_leave` - Callback invoked after the open() call (read-only)
///
/// # Errors
/// Returns `Error::InvalidArg` if the name contains null bytes, or
/// `Error::RegisterFailed` if registration fails.
pub fn register_file_open(
    name: &str,
    on_enter: Option<FileOpenEnterFn>,
    on_leave: Option<FileOpenLeaveFn>,
) -> Result<FilterHandle, Error> {
    let name_cstr = std::ffi::CString::new(name).map_err(|_| Error::InvalidArg)?;

    // Store function pointers in static storage for the C callbacks
    let user_data = Box::new((on_enter, on_leave));
    let user_data_ptr = Box::into_raw(user_data) as *mut c_void;

    let handle = unsafe {
        qcontrol_register_file_open_filter(
            name_cstr.as_ptr(),
            on_enter.map(|_| file_open_enter_wrapper as RawFileOpenFilterFn),
            on_leave.map(|_| file_open_leave_wrapper as RawFileOpenFilterFn),
            user_data_ptr,
        )
    };

    if handle == 0 {
        // Clean up on failure
        unsafe {
            drop(Box::from_raw(
                user_data_ptr as *mut (Option<FileOpenEnterFn>, Option<FileOpenLeaveFn>),
            ));
        }
        Err(Error::RegisterFailed)
    } else {
        Ok(FilterHandle(handle))
    }
}

/// Register a read filter using simple functions.
///
/// # Arguments
/// * `name` - A unique name for this filter
/// * `on_enter` - Callback invoked before the read() call
/// * `on_leave` - Callback invoked after the read() call (read-only)
///
/// # Errors
/// Returns `Error::InvalidArg` if the name contains null bytes, or
/// `Error::RegisterFailed` if registration fails.
pub fn register_file_read(
    name: &str,
    on_enter: Option<FileReadEnterFn>,
    on_leave: Option<FileReadLeaveFn>,
) -> Result<FilterHandle, Error> {
    let name_cstr = std::ffi::CString::new(name).map_err(|_| Error::InvalidArg)?;
    let user_data = Box::new((on_enter, on_leave));
    let user_data_ptr = Box::into_raw(user_data) as *mut c_void;

    let handle = unsafe {
        qcontrol_register_file_read_filter(
            name_cstr.as_ptr(),
            on_enter.map(|_| file_read_enter_wrapper as RawFileReadFilterFn),
            on_leave.map(|_| file_read_leave_wrapper as RawFileReadFilterFn),
            user_data_ptr,
        )
    };

    if handle == 0 {
        unsafe {
            drop(Box::from_raw(
                user_data_ptr as *mut (Option<FileReadEnterFn>, Option<FileReadLeaveFn>),
            ));
        }
        Err(Error::RegisterFailed)
    } else {
        Ok(FilterHandle(handle))
    }
}

/// Register a write filter using simple functions.
///
/// # Arguments
/// * `name` - A unique name for this filter
/// * `on_enter` - Callback invoked before the write() call
/// * `on_leave` - Callback invoked after the write() call (read-only)
///
/// # Errors
/// Returns `Error::InvalidArg` if the name contains null bytes, or
/// `Error::RegisterFailed` if registration fails.
pub fn register_file_write(
    name: &str,
    on_enter: Option<FileWriteEnterFn>,
    on_leave: Option<FileWriteLeaveFn>,
) -> Result<FilterHandle, Error> {
    let name_cstr = std::ffi::CString::new(name).map_err(|_| Error::InvalidArg)?;
    let user_data = Box::new((on_enter, on_leave));
    let user_data_ptr = Box::into_raw(user_data) as *mut c_void;

    let handle = unsafe {
        qcontrol_register_file_write_filter(
            name_cstr.as_ptr(),
            on_enter.map(|_| file_write_enter_wrapper as RawFileWriteFilterFn),
            on_leave.map(|_| file_write_leave_wrapper as RawFileWriteFilterFn),
            user_data_ptr,
        )
    };

    if handle == 0 {
        unsafe {
            drop(Box::from_raw(
                user_data_ptr as *mut (Option<FileWriteEnterFn>, Option<FileWriteLeaveFn>),
            ));
        }
        Err(Error::RegisterFailed)
    } else {
        Ok(FilterHandle(handle))
    }
}

/// Register a close filter using simple functions.
///
/// # Arguments
/// * `name` - A unique name for this filter
/// * `on_enter` - Callback invoked before the close() call
/// * `on_leave` - Callback invoked after the close() call (read-only)
///
/// # Errors
/// Returns `Error::InvalidArg` if the name contains null bytes, or
/// `Error::RegisterFailed` if registration fails.
pub fn register_file_close(
    name: &str,
    on_enter: Option<FileCloseEnterFn>,
    on_leave: Option<FileCloseLeaveFn>,
) -> Result<FilterHandle, Error> {
    let name_cstr = std::ffi::CString::new(name).map_err(|_| Error::InvalidArg)?;
    let user_data = Box::new((on_enter, on_leave));
    let user_data_ptr = Box::into_raw(user_data) as *mut c_void;

    let handle = unsafe {
        qcontrol_register_file_close_filter(
            name_cstr.as_ptr(),
            on_enter.map(|_| file_close_enter_wrapper as RawFileCloseFilterFn),
            on_leave.map(|_| file_close_leave_wrapper as RawFileCloseFilterFn),
            user_data_ptr,
        )
    };

    if handle == 0 {
        unsafe {
            drop(Box::from_raw(
                user_data_ptr as *mut (Option<FileCloseEnterFn>, Option<FileCloseLeaveFn>),
            ));
        }
        Err(Error::RegisterFailed)
    } else {
        Ok(FilterHandle(handle))
    }
}

/// Unregister a previously registered filter.
///
/// # Arguments
/// * `handle` - The handle returned from a registration function
///
/// # Errors
/// Returns `Error::InvalidArg` if the handle is invalid.
pub fn unregister(handle: FilterHandle) -> Result<(), Error> {
    let result = unsafe { qcontrol_unregister_filter(handle.0) };
    if result == 0 {
        Ok(())
    } else {
        Err(Error::InvalidArg)
    }
}

// ============================================================================
// Plugin Macro
// ============================================================================

/// Declare a qcontrol plugin with fallible initialization.
///
/// This macro generates the required C exports automatically. The initialization
/// closure should return `Result<(), Error>` to support proper error handling.
///
/// # Example
///
/// ```rust,ignore
/// use qcontrol::{plugin, register_file_open, FileOpenContext, FilterResult, Error};
///
/// fn log_open(ctx: &FileOpenContext) -> FilterResult {
///     eprintln!("open({}) = {}", ctx.path(), ctx.result());
///     FilterResult::Continue
/// }
///
/// plugin!(|| -> Result<(), Error> {
///     register_file_open("my_logger", None, Some(log_open))?;
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! plugin {
    ($init:expr) => {
        #[no_mangle]
        pub extern "C" fn qcontrol_plugin_init() -> i32 {
            let init_fn: fn() -> Result<(), $crate::Error> = $init;
            match init_fn() {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }

        #[no_mangle]
        pub extern "C" fn qcontrol_plugin_cleanup() {}
    };
}

// ============================================================================
// Internal: Raw C Types and FFI (from bindgen-generated bindings)
// ============================================================================

// Re-export types from bindgen for internal use
type RawFileOpenCtx = ffi::qcontrol_file_open_ctx_t;
type RawFileReadCtx = ffi::qcontrol_file_read_ctx_t;
type RawFileWriteCtx = ffi::qcontrol_file_write_ctx_t;
type RawFileCloseCtx = ffi::qcontrol_file_close_ctx_t;

// Use the constant from C headers
const MAX_PATH: usize = ffi::QCONTROL_MAX_PATH as usize;

type RawFileOpenFilterFn = extern "C" fn(*mut RawFileOpenCtx) -> ffi::qcontrol_status_t;
type RawFileReadFilterFn = extern "C" fn(*mut RawFileReadCtx) -> ffi::qcontrol_status_t;
type RawFileWriteFilterFn = extern "C" fn(*mut RawFileWriteCtx) -> ffi::qcontrol_status_t;
type RawFileCloseFilterFn = extern "C" fn(*mut RawFileCloseCtx) -> ffi::qcontrol_status_t;

extern "C" {
    fn qcontrol_register_file_open_filter(
        name: *const c_char,
        on_enter: Option<RawFileOpenFilterFn>,
        on_leave: Option<RawFileOpenFilterFn>,
        user_data: *mut c_void,
    ) -> ffi::qcontrol_filter_handle_t;

    fn qcontrol_register_file_read_filter(
        name: *const c_char,
        on_enter: Option<RawFileReadFilterFn>,
        on_leave: Option<RawFileReadFilterFn>,
        user_data: *mut c_void,
    ) -> ffi::qcontrol_filter_handle_t;

    fn qcontrol_register_file_write_filter(
        name: *const c_char,
        on_enter: Option<RawFileWriteFilterFn>,
        on_leave: Option<RawFileWriteFilterFn>,
        user_data: *mut c_void,
    ) -> ffi::qcontrol_filter_handle_t;

    fn qcontrol_register_file_close_filter(
        name: *const c_char,
        on_enter: Option<RawFileCloseFilterFn>,
        on_leave: Option<RawFileCloseFilterFn>,
        user_data: *mut c_void,
    ) -> ffi::qcontrol_filter_handle_t;

    fn qcontrol_unregister_filter(handle: ffi::qcontrol_filter_handle_t) -> i32;
}

fn result_to_c(r: FilterResult) -> ffi::qcontrol_status_t {
    match r {
        FilterResult::Continue => ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE,
        FilterResult::Modify => ffi::qcontrol_status_t::QCONTROL_STATUS_MODIFY,
        FilterResult::Block => ffi::qcontrol_status_t::QCONTROL_STATUS_BLOCK,
    }
}

// Wrapper functions that convert between C and Rust types
extern "C" fn file_open_enter_wrapper(ctx: *mut RawFileOpenCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileOpenEnterFn>, Option<FileOpenLeaveFn>);
    let (on_enter, _) = unsafe { &*user_data };

    if let Some(f) = on_enter {
        let mut wrapper = FileOpenContext { inner: ctx_ref };
        result_to_c(f(&mut wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_open_leave_wrapper(ctx: *mut RawFileOpenCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileOpenEnterFn>, Option<FileOpenLeaveFn>);
    let (_, on_leave) = unsafe { &*user_data };

    if let Some(f) = on_leave {
        let wrapper = FileOpenContext { inner: ctx_ref };
        result_to_c(f(&wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_read_enter_wrapper(ctx: *mut RawFileReadCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileReadEnterFn>, Option<FileReadLeaveFn>);
    let (on_enter, _) = unsafe { &*user_data };

    if let Some(f) = on_enter {
        let mut wrapper = FileReadContext { inner: ctx_ref };
        result_to_c(f(&mut wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_read_leave_wrapper(ctx: *mut RawFileReadCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileReadEnterFn>, Option<FileReadLeaveFn>);
    let (_, on_leave) = unsafe { &*user_data };

    if let Some(f) = on_leave {
        let wrapper = FileReadContext { inner: ctx_ref };
        result_to_c(f(&wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_write_enter_wrapper(ctx: *mut RawFileWriteCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileWriteEnterFn>, Option<FileWriteLeaveFn>);
    let (on_enter, _) = unsafe { &*user_data };

    if let Some(f) = on_enter {
        let mut wrapper = FileWriteContext { inner: ctx_ref };
        result_to_c(f(&mut wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_write_leave_wrapper(ctx: *mut RawFileWriteCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileWriteEnterFn>, Option<FileWriteLeaveFn>);
    let (_, on_leave) = unsafe { &*user_data };

    if let Some(f) = on_leave {
        let wrapper = FileWriteContext { inner: ctx_ref };
        result_to_c(f(&wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_close_enter_wrapper(ctx: *mut RawFileCloseCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileCloseEnterFn>, Option<FileCloseLeaveFn>);
    let (on_enter, _) = unsafe { &*user_data };

    if let Some(f) = on_enter {
        let mut wrapper = FileCloseContext { inner: ctx_ref };
        result_to_c(f(&mut wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}

extern "C" fn file_close_leave_wrapper(ctx: *mut RawFileCloseCtx) -> ffi::qcontrol_status_t {
    let ctx_ref = unsafe { &mut *ctx };
    let user_data = ctx_ref.user_data as *const (Option<FileCloseEnterFn>, Option<FileCloseLeaveFn>);
    let (_, on_leave) = unsafe { &*user_data };

    if let Some(f) = on_leave {
        let wrapper = FileCloseContext { inner: ctx_ref };
        result_to_c(f(&wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}
