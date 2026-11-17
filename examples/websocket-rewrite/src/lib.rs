//! WebSocket rewrite plugin example.
//!
//! Demonstrates the post-upgrade WebSocket Rust SDK surface:
//! - stream-scoped state created from `on_websocket_open`
//! - correlation through `connection_id` and `exchange_id`
//! - logical message mutation on both send and receive callbacks

use std::cell::Cell;

use qcontrol::prelude::*;

/// Tracks one upgraded WebSocket stream and how many messages were rewritten.
struct WebSocketState {
    connection_id: u64,
    exchange_id: u64,
    rewritten: Cell<u64>,
}

/// Start one stream-scoped state record after the upgrade handshake succeeds.
fn on_websocket_open(ev: &WebSocketOpenEvent) -> StreamAction {
    let ctx = ev.ctx();
    let _ = (ev.subprotocol(), ev.extensions());

    StreamAction::State(Box::new(WebSocketState {
        connection_id: ctx.connection_id(),
        exchange_id: ctx.exchange_id(),
        rewritten: Cell::new(0),
    }))
}

/// Rewrite one logical text message when it matches the expected payload.
fn rewrite_message(
    state: &WebSocketState,
    ev: &mut WebSocketMessageEvent,
    expected: &str,
    replacement: &str,
) -> StreamAction {
    let ctx = ev.ctx();
    let _ = (
        state.connection_id,
        state.exchange_id,
        ctx.connection_id(),
        ctx.exchange_id(),
        ctx.protocol(),
    );

    if ev.kind() != WebSocketMessageKind::Text || !ev.can_mutate() {
        return StreamAction::Pass;
    }

    let Some(text) = ev.text() else {
        return StreamAction::Pass;
    };
    if text != expected {
        return StreamAction::Pass;
    }

    state.rewritten.set(state.rewritten.get() + 1);
    let _ = ev.set_text(replacement);
    StreamAction::Pass
}

/// Rewrite one outgoing logical message payload.
fn on_websocket_send(state: PluginState, ev: &mut WebSocketMessageEvent) -> StreamAction {
    let Some(state) = state.downcast_ref::<WebSocketState>() else {
        return StreamAction::Pass;
    };
    rewrite_message(state, ev, "ping", "ping from qcontrol")
}

/// Rewrite one incoming logical message payload.
fn on_websocket_recv(state: PluginState, ev: &mut WebSocketMessageEvent) -> StreamAction {
    let Some(state) = state.downcast_ref::<WebSocketState>() else {
        return StreamAction::Pass;
    };
    rewrite_message(state, ev, "hello", "hello from qcontrol")
}

/// Observe stream closure and keep the example state lifecycle explicit.
fn on_websocket_close(state: PluginState, ev: &WebSocketCloseEvent) {
    if let Some(state) = state.downcast_ref::<WebSocketState>() {
        let _ = (
            state.connection_id,
            state.exchange_id,
            state.rewritten.get(),
            ev.ctx().connection_id(),
            ev.ctx().exchange_id(),
            ev.direction(),
            ev.code(),
            ev.clean(),
            ev.reason(),
        );
    }
}

export_plugin!(PluginBuilder::new("rust_websocket_rewrite")
    .on_websocket_open(on_websocket_open)
    .on_websocket_send(on_websocket_send)
    .on_websocket_recv(on_websocket_recv)
    .on_websocket_close(on_websocket_close));
