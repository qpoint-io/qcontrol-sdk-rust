//! Internal FFI bindings (not part of public API)

use std::ffi::{c_char, c_void};

// Include bindgen-generated bindings from C headers
// This ensures C headers are the single source of truth for ABI types
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Re-export types from bindgen for internal use
pub(crate) use bindings::qcontrol_filter_handle_t;
pub(crate) use bindings::qcontrol_status_t;

pub(crate) type RawFileOpenCtx = bindings::qcontrol_file_open_ctx_t;
pub(crate) type RawFileReadCtx = bindings::qcontrol_file_read_ctx_t;
pub(crate) type RawFileWriteCtx = bindings::qcontrol_file_write_ctx_t;
pub(crate) type RawFileCloseCtx = bindings::qcontrol_file_close_ctx_t;

// Use the constant from C headers
pub(crate) const MAX_PATH: usize = bindings::QCONTROL_MAX_PATH as usize;

pub(crate) type RawFileOpenFilterFn = extern "C" fn(*mut RawFileOpenCtx) -> qcontrol_status_t;
pub(crate) type RawFileReadFilterFn = extern "C" fn(*mut RawFileReadCtx) -> qcontrol_status_t;
pub(crate) type RawFileWriteFilterFn = extern "C" fn(*mut RawFileWriteCtx) -> qcontrol_status_t;
pub(crate) type RawFileCloseFilterFn = extern "C" fn(*mut RawFileCloseCtx) -> qcontrol_status_t;

extern "C" {
    pub(crate) fn qcontrol_register_file_open_filter(
        name: *const c_char,
        on_enter: Option<RawFileOpenFilterFn>,
        on_leave: Option<RawFileOpenFilterFn>,
        user_data: *mut c_void,
    ) -> qcontrol_filter_handle_t;

    pub(crate) fn qcontrol_register_file_read_filter(
        name: *const c_char,
        on_enter: Option<RawFileReadFilterFn>,
        on_leave: Option<RawFileReadFilterFn>,
        user_data: *mut c_void,
    ) -> qcontrol_filter_handle_t;

    pub(crate) fn qcontrol_register_file_write_filter(
        name: *const c_char,
        on_enter: Option<RawFileWriteFilterFn>,
        on_leave: Option<RawFileWriteFilterFn>,
        user_data: *mut c_void,
    ) -> qcontrol_filter_handle_t;

    pub(crate) fn qcontrol_register_file_close_filter(
        name: *const c_char,
        on_enter: Option<RawFileCloseFilterFn>,
        on_leave: Option<RawFileCloseFilterFn>,
        user_data: *mut c_void,
    ) -> qcontrol_filter_handle_t;

    pub(crate) fn qcontrol_unregister_filter(handle: qcontrol_filter_handle_t) -> i32;
}

use crate::FilterResult;

pub(crate) fn result_to_c(r: FilterResult) -> qcontrol_status_t {
    match r {
        FilterResult::Continue => qcontrol_status_t::QCONTROL_STATUS_CONTINUE,
        FilterResult::Modify => qcontrol_status_t::QCONTROL_STATUS_MODIFY,
        FilterResult::Block => qcontrol_status_t::QCONTROL_STATUS_BLOCK,
    }
}
