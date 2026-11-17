//! Network action result types
//!
//! Defines the return types for network callbacks.

use crate::ffi;
use crate::net::{NetSession, SessionState};
use std::ffi::c_void;

/// Result returned from `on_net_connect` callback.
///
/// Determines how the agent handles the outbound connection.
#[derive(Debug)]
pub enum ConnectResult {
    /// No interception - continue normally.
    Pass,
    /// Block the connection with EACCES.
    Block,
    /// Block the connection with a specific errno.
    BlockErrno(i32),
    /// Intercept with a session configuration for send/recv transforms.
    Session(NetSession),
    /// Track state only, no transforms.
    State(*mut c_void),
}

impl ConnectResult {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_net_action_t {
        match self {
            ConnectResult::Pass => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            ConnectResult::Block => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            ConnectResult::BlockErrno(errno) => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: errno },
            },
            ConnectResult::Session(session) => {
                let ffi_session = session.into_ffi();
                ffi::qcontrol_net_action_t {
                    type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_SESSION,
                    __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 {
                        session: ffi_session,
                    },
                }
            }
            ConnectResult::State(ptr) => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_STATE,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 {
                    state: if ptr.is_null() {
                        std::ptr::null_mut()
                    } else {
                        SessionState::from_raw_state(ptr)
                    },
                },
            },
        }
    }
}

/// Result returned from `on_net_accept` callback.
///
/// Determines how the agent handles the inbound connection.
#[derive(Debug)]
pub enum AcceptResult {
    /// No interception - continue normally.
    Pass,
    /// Block the connection with EACCES.
    Block,
    /// Block the connection with a specific errno.
    BlockErrno(i32),
    /// Intercept with a session configuration for send/recv transforms.
    Session(NetSession),
    /// Track state only, no transforms.
    State(*mut c_void),
}

impl AcceptResult {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_net_action_t {
        match self {
            AcceptResult::Pass => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            AcceptResult::Block => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            AcceptResult::BlockErrno(errno) => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: errno },
            },
            AcceptResult::Session(session) => {
                let ffi_session = session.into_ffi();
                ffi::qcontrol_net_action_t {
                    type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_SESSION,
                    __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 {
                        session: ffi_session,
                    },
                }
            }
            AcceptResult::State(ptr) => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_STATE,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 {
                    state: if ptr.is_null() {
                        std::ptr::null_mut()
                    } else {
                        SessionState::from_raw_state(ptr)
                    },
                },
            },
        }
    }
}

/// Result returned from `on_net_send`, `on_net_recv` callbacks.
///
/// These callbacks can observe and optionally block operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetAction {
    /// Continue normally.
    Pass,
    /// Block the operation with EACCES.
    Block,
    /// Block the operation with a specific errno.
    BlockErrno(i32),
}

impl NetAction {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_net_action_t {
        match self {
            NetAction::Pass => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            NetAction::Block => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: 0 },
            },
            NetAction::BlockErrno(errno) => ffi::qcontrol_net_action_t {
                type_: ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_net_action__bindgen_ty_1 { errno_val: errno },
            },
        }
    }
}

/// Result returned from `on_net_tls`, `on_net_domain`, and `on_net_protocol`.
///
/// These callbacks run after a connection session already exists and can keep it
/// alive or terminate it once stronger identity evidence is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetPhaseAction {
    /// Keep the tracked connection open.
    Pass,
    /// Terminate the tracked connection with the default denied behavior.
    Block,
    /// Terminate the tracked connection with a specific errno.
    BlockErrno(i32),
}

impl NetPhaseAction {
    /// Convert to FFI phase action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_net_phase_action_t {
        match self {
            NetPhaseAction::Pass => ffi::qcontrol_net_phase_action_t {
                type_: ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_PASS,
                __bindgen_anon_1: ffi::qcontrol_net_phase_action__bindgen_ty_1 { errno_val: 0 },
            },
            NetPhaseAction::Block => ffi::qcontrol_net_phase_action_t {
                type_: ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_BLOCK,
                __bindgen_anon_1: ffi::qcontrol_net_phase_action__bindgen_ty_1 { errno_val: 0 },
            },
            NetPhaseAction::BlockErrno(errno) => ffi::qcontrol_net_phase_action_t {
                type_: ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_BLOCK_ERRNO,
                __bindgen_anon_1: ffi::qcontrol_net_phase_action__bindgen_ty_1 { errno_val: errno },
            },
        }
    }
}

/// Connection direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetDirection {
    /// Outbound connection (connect).
    Outbound,
    /// Inbound connection (accept).
    Inbound,
}

impl From<ffi::qcontrol_net_direction_t> for NetDirection {
    fn from(dir: ffi::qcontrol_net_direction_t) -> Self {
        if dir == ffi::qcontrol_net_direction_t_QCONTROL_NET_INBOUND {
            NetDirection::Inbound
        } else {
            NetDirection::Outbound
        }
    }
}

impl From<NetDirection> for ffi::qcontrol_net_direction_t {
    fn from(dir: NetDirection) -> Self {
        match dir {
            NetDirection::Outbound => ffi::qcontrol_net_direction_t_QCONTROL_NET_OUTBOUND,
            NetDirection::Inbound => ffi::qcontrol_net_direction_t_QCONTROL_NET_INBOUND,
        }
    }
}

/// Per-connection tunnel-versus-intercept preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetInspectionMode {
    /// Preserve the host's default handling.
    Default,
    /// Prefer raw tunneling without interception.
    Tunnel,
    /// Prefer active interception when the host supports it.
    Intercept,
}

impl From<NetInspectionMode> for ffi::qcontrol_net_inspection_mode_t {
    fn from(mode: NetInspectionMode) -> Self {
        match mode {
            NetInspectionMode::Default => {
                ffi::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_DEFAULT
            }
            NetInspectionMode::Tunnel => {
                ffi::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_TUNNEL
            }
            NetInspectionMode::Intercept => {
                ffi::qcontrol_net_inspection_mode_t_QCONTROL_NET_INSPECTION_INTERCEPT
            }
        }
    }
}
