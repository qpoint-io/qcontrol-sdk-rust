//! File action result types
//!
//! Defines the return types for file callbacks.

use crate::ffi;
use crate::file::FileSession;

/// Result returned from `on_file_open` callback.
///
/// Determines how the agent handles the opened file.
#[derive(Debug)]
pub enum FileOpenResult {
    /// No interception - continue normally without tracking this file.
    Pass,
    /// Block the open operation with EACCES.
    Block,
    /// Block the open operation with a specific errno.
    BlockErrno(i32),
    /// Intercept with a session configuration for read/write transforms.
    Session(FileSession),
}

impl FileOpenResult {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_file_action_t {
        match self {
            FileOpenResult::Pass => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: 0 },
            },
            FileOpenResult::Block => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: 0 },
            },
            FileOpenResult::BlockErrno(errno) => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: errno },
            },
            FileOpenResult::Session(session) => {
                let ffi_session = session.into_ffi();
                ffi::qcontrol_file_action_t {
                    type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_SESSION,
                    __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 {
                        session: ffi_session,
                    },
                }
            }
        }
    }
}

/// Result returned from `on_file_read`, `on_file_write` callbacks.
///
/// These callbacks can observe and optionally block operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    /// Continue normally.
    Pass,
    /// Block the operation with EACCES.
    Block,
    /// Block the operation with a specific errno.
    BlockErrno(i32),
}

impl FileAction {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_file_action_t {
        match self {
            FileAction::Pass => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: 0 },
            },
            FileAction::Block => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: 0 },
            },
            FileAction::BlockErrno(errno) => ffi::qcontrol_file_action_t {
                type_: ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_file_action__bindgen_ty_1 { errno_val: errno },
            },
        }
    }
}
