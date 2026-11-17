//! Shared derived-stream action type.
//!
//! SSE and WebSocket callback families use one common action surface: pass the
//! current record, terminate the stream, or attach one stream-scoped state
//! value for later callbacks.

use std::any::Any;
use std::ffi::c_void;

use crate::ffi;
use crate::stream::StreamState;

/// Result returned from one derived-stream callback.
pub enum StreamAction {
    /// Continue forwarding the current record normally.
    Pass,
    /// Terminate the derived stream.
    Block,
    /// Attach or replace the stream-scoped plugin state.
    State(Box<dyn Any + Send>),
}

impl StreamAction {
    /// Convert the Rust action into the C ABI struct.
    #[doc(hidden)]
    pub fn to_ffi(self) -> ffi::qcontrol_stream_action_t {
        match self {
            StreamAction::Pass => ffi::qcontrol_stream_action_t {
                type_: ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                state: std::ptr::null_mut(),
            },
            StreamAction::Block => ffi::qcontrol_stream_action_t {
                type_: ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_BLOCK,
                state: std::ptr::null_mut(),
            },
            StreamAction::State(user_state) => {
                let stream_state = StreamState {
                    user_state: Some(user_state),
                };
                ffi::qcontrol_stream_action_t {
                    type_: ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_STATE,
                    state: Box::into_raw(Box::new(stream_state)) as *mut c_void,
                }
            }
        }
    }
}
