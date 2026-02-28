//! Filter registration functions and callback types

use std::ffi::c_void;

use crate::file::{FileCloseContext, FileOpenContext, FileReadContext, FileWriteContext};
use crate::ffi::{
    self, qcontrol_register_file_close_filter, qcontrol_register_file_open_filter,
    qcontrol_register_file_read_filter, qcontrol_register_file_write_filter,
    qcontrol_unregister_filter, result_to_c, RawFileCloseCtx, RawFileCloseFilterFn,
    RawFileOpenCtx, RawFileOpenFilterFn, RawFileReadCtx, RawFileReadFilterFn, RawFileWriteCtx,
    RawFileWriteFilterFn,
};
use crate::types::{Error, FilterHandle, FilterResult};

// ============================================================================
// Callback Type Aliases
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

// ============================================================================
// Registration Functions
// ============================================================================

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
// Internal Wrapper Functions
// ============================================================================

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
    let user_data =
        ctx_ref.user_data as *const (Option<FileWriteEnterFn>, Option<FileWriteLeaveFn>);
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
    let user_data =
        ctx_ref.user_data as *const (Option<FileWriteEnterFn>, Option<FileWriteLeaveFn>);
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
    let user_data =
        ctx_ref.user_data as *const (Option<FileCloseEnterFn>, Option<FileCloseLeaveFn>);
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
    let user_data =
        ctx_ref.user_data as *const (Option<FileCloseEnterFn>, Option<FileCloseLeaveFn>);
    let (_, on_leave) = unsafe { &*user_data };

    if let Some(f) = on_leave {
        let wrapper = FileCloseContext { inner: ctx_ref };
        result_to_c(f(&wrapper))
    } else {
        ffi::qcontrol_status_t::QCONTROL_STATUS_CONTINUE
    }
}
