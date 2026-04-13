//! HTTP action result types
//!
//! Two action types:
//! - `HttpRequestAction`: returned from `on_http_request` — can attach per-exchange state
//! - `HttpAction`: returned from all other action-returning callbacks — Pass or Block only
//!
//! The runtime only supports attaching state from the initial request callback.
//! Returning State from later callbacks would silently overwrite the existing
//! state pointer without cleaning up the previous value.

use std::any::Any;
use std::ffi::c_void;

use crate::ffi;
use crate::http::HttpState;

/// Result returned from `on_http_request`.
///
/// This is the only callback that can attach per-exchange state via `State`.
pub enum HttpRequestAction {
    /// Observe only — continue normally.
    Pass,
    /// Block the exchange.
    Block,
    /// Track per-exchange state. The state will be passed to subsequent
    /// callbacks for this exchange and cleaned up on exchange close.
    State(Box<dyn Any + Send>),
}

impl HttpRequestAction {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_http_action_t {
        match self {
            HttpRequestAction::Pass => pass_action(),
            HttpRequestAction::Block => block_action(),
            HttpRequestAction::State(user_state) => {
                let http_state = HttpState {
                    user_state: Some(user_state),
                };
                let state_ptr = Box::into_raw(Box::new(http_state)) as *mut c_void;
                ffi::qcontrol_http_action_t {
                    type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_STATE,
                    __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 { state: state_ptr },
                }
            }
        }
    }
}

/// Result returned from HTTP body, trailers, response, and other
/// action-returning callbacks.
///
/// State attachment is not available here — use `HttpRequestAction::State`
/// in `on_http_request` to attach per-exchange state.
pub enum HttpAction {
    /// Continue normally.
    Pass,
    /// Block the exchange.
    Block,
}

impl HttpAction {
    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_http_action_t {
        match self {
            HttpAction::Pass => pass_action(),
            HttpAction::Block => block_action(),
        }
    }
}

fn pass_action() -> ffi::qcontrol_http_action_t {
    ffi::qcontrol_http_action_t {
        type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
        __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 {
            state: std::ptr::null_mut(),
        },
    }
}

fn block_action() -> ffi::qcontrol_http_action_t {
    ffi::qcontrol_http_action_t {
        type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_BLOCK,
        __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 {
            state: std::ptr::null_mut(),
        },
    }
}
