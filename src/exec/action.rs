//! Exec action result types
//!
//! Defines the return types for exec callbacks.

use crate::exec::ExecSession;
use crate::ffi;
use std::ffi::c_void;

/// Result returned from `on_exec` callback.
///
/// Determines how the agent handles the exec operation.
#[derive(Debug)]
pub enum ExecResult {
    /// No interception - continue normally.
    Pass,
    /// Block the exec with EACCES.
    Block,
    /// Block the exec with a specific errno.
    BlockErrno(i32),
    /// Intercept with a session configuration for I/O transforms.
    Session(ExecSession),
    /// Track state only, no transforms.
    State(*mut c_void),
}

impl ExecResult {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_exec_action_t {
        match self {
            ExecResult::Pass => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: 0 },
            },
            ExecResult::Block => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: 0 },
            },
            ExecResult::BlockErrno(errno) => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: errno },
            },
            ExecResult::Session(session) => {
                let ffi_session = session.into_ffi();
                ffi::qcontrol_exec_action_t {
                    type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_SESSION,
                    __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 {
                        session: ffi_session,
                    },
                }
            }
            ExecResult::State(ptr) => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_STATE,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { state: ptr },
            },
        }
    }
}

/// Result returned from `on_exec_stdin`, `on_exec_stdout`, `on_exec_stderr` callbacks.
///
/// These callbacks can observe and optionally block operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecAction {
    /// Continue normally.
    Pass,
    /// Block the operation with EACCES.
    Block,
    /// Block the operation with a specific errno.
    BlockErrno(i32),
}

impl ExecAction {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_exec_action_t {
        match self {
            ExecAction::Pass => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: 0 },
            },
            ExecAction::Block => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: 0 },
            },
            ExecAction::BlockErrno(errno) => ffi::qcontrol_exec_action_t {
                type_: ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_exec_action__bindgen_ty_1 { errno_val: errno },
            },
        }
    }
}
