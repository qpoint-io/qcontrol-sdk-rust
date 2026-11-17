//! WebSocket operation types.
//!
//! WebSocket callbacks begin after the HTTP upgrade handshake succeeds. The
//! host owns frame parsing and message reassembly before plugins see logical
//! text or binary messages.

mod event;

pub use event::{
    WebSocketCloseEvent, WebSocketMessageEvent, WebSocketMessageKind, WebSocketOpenEvent,
};

pub use crate::state::{FileState, PluginState};

/// Callback type for the start of one upgraded WebSocket stream.
pub type WebSocketOpenFn = fn(&WebSocketOpenEvent) -> crate::stream::StreamAction;

/// Callback type for one logical WebSocket send or receive message.
pub type WebSocketMessageFn =
    fn(PluginState, &mut WebSocketMessageEvent) -> crate::stream::StreamAction;

/// Callback type for the end of one upgraded WebSocket stream.
pub type WebSocketCloseFn = fn(PluginState, &WebSocketCloseEvent);
