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

/// Body scheduling mode requested from one request or response head callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpBodyMode {
    /// Preserve the host's default body scheduling for this message.
    Default,
    /// Deliver decoded body callbacks incrementally as chunks arrive.
    Stream,
    /// Buffer the decoded logical body before running body callbacks.
    Buffer,
}

impl HttpBodyMode {
    /// Convert the Rust body mode into the C ABI enum.
    fn to_ffi(self) -> ffi::qcontrol_http_body_mode_t {
        match self {
            HttpBodyMode::Default => ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
            HttpBodyMode::Stream => ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_STREAM,
            HttpBodyMode::Buffer => ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_BUFFER,
        }
    }
}

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
    /// Observe only and request a specific body scheduling mode.
    PassWithBodyMode(HttpBodyMode),
    /// Track per-exchange state and request a specific body scheduling mode.
    StateWithBodyMode(Box<dyn Any + Send>, HttpBodyMode),
}

impl HttpRequestAction {
    /// Request a specific body scheduling mode for this exchange's request body.
    pub fn with_body_mode(self, body_mode: HttpBodyMode) -> Self {
        match self {
            HttpRequestAction::Pass => HttpRequestAction::PassWithBodyMode(body_mode),
            HttpRequestAction::State(user_state) => {
                HttpRequestAction::StateWithBodyMode(user_state, body_mode)
            }
            HttpRequestAction::PassWithBodyMode(_) => {
                HttpRequestAction::PassWithBodyMode(body_mode)
            }
            HttpRequestAction::StateWithBodyMode(user_state, _) => {
                HttpRequestAction::StateWithBodyMode(user_state, body_mode)
            }
            HttpRequestAction::Block => HttpRequestAction::Block,
        }
    }

    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_http_action_t {
        match self {
            HttpRequestAction::Pass => pass_action(),
            HttpRequestAction::Block => block_action(),
            HttpRequestAction::State(user_state) => state_action(user_state, HttpBodyMode::Default),
            HttpRequestAction::PassWithBodyMode(body_mode) => pass_action_with_body_mode(body_mode),
            HttpRequestAction::StateWithBodyMode(user_state, body_mode) => {
                state_action(user_state, body_mode)
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
    /// Continue normally and request a specific body scheduling mode.
    PassWithBodyMode(HttpBodyMode),
}

impl HttpAction {
    /// Request a specific body scheduling mode for this message.
    ///
    /// Request head callbacks apply the selected mode to the request body.
    /// Response head callbacks apply the selected mode to the response body.
    pub fn with_body_mode(self, body_mode: HttpBodyMode) -> Self {
        match self {
            HttpAction::Pass => HttpAction::PassWithBodyMode(body_mode),
            HttpAction::PassWithBodyMode(_) => HttpAction::PassWithBodyMode(body_mode),
            HttpAction::Block => HttpAction::Block,
        }
    }

    /// Convert to FFI action type.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_http_action_t {
        match self {
            HttpAction::Pass => pass_action(),
            HttpAction::Block => block_action(),
            HttpAction::PassWithBodyMode(body_mode) => pass_action_with_body_mode(body_mode),
        }
    }
}

fn pass_action() -> ffi::qcontrol_http_action_t {
    ffi::qcontrol_http_action_t {
        type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
        body_mode: ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
        __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 {
            state: std::ptr::null_mut(),
        },
    }
}

fn pass_action_with_body_mode(body_mode: HttpBodyMode) -> ffi::qcontrol_http_action_t {
    let mut action = pass_action();
    action.body_mode = body_mode.to_ffi();
    action
}

fn state_action(
    user_state: Box<dyn Any + Send>,
    body_mode: HttpBodyMode,
) -> ffi::qcontrol_http_action_t {
    let http_state = HttpState {
        user_state: Some(user_state),
    };
    let state_ptr = Box::into_raw(Box::new(http_state)) as *mut c_void;
    ffi::qcontrol_http_action_t {
        type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_STATE,
        body_mode: body_mode.to_ffi(),
        __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 { state: state_ptr },
    }
}

fn block_action() -> ffi::qcontrol_http_action_t {
    ffi::qcontrol_http_action_t {
        type_: ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_BLOCK,
        body_mode: ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
        __bindgen_anon_1: ffi::qcontrol_http_action__bindgen_ty_1 {
            state: std::ptr::null_mut(),
        },
    }
}
