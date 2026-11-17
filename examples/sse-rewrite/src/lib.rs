//! SSE rewrite plugin example.
//!
//! Demonstrates the parsed SSE Rust SDK surface:
//! - stream-scoped state created from `on_sse_open`
//! - correlation through `connection_id` and `exchange_id`
//! - in-place mutation of parsed `event`, `id`, `retry`, and `data` fields

use std::cell::Cell;

use qcontrol::prelude::*;

/// Tracks one SSE stream and counts how many events were rewritten.
struct SseState {
    connection_id: u64,
    exchange_id: u64,
    rewritten: Cell<u64>,
}

/// Start one stream-scoped state record when the HTTP response becomes SSE.
fn on_sse_open(ev: &SseOpenEvent) -> StreamAction {
    let ctx = ev.ctx();
    let _ = ev.content_type();

    StreamAction::State(Box::new(SseState {
        connection_id: ctx.connection_id(),
        exchange_id: ctx.exchange_id(),
        rewritten: Cell::new(0),
    }))
}

/// Rewrite one parsed SSE event record in place.
fn on_sse_event(state: PluginState, ev: &mut SseEventRecord) -> StreamAction {
    let Some(state) = state.downcast_ref::<SseState>() else {
        return StreamAction::Pass;
    };

    let ctx = ev.ctx();
    let _ = (
        state.connection_id,
        state.exchange_id,
        ctx.connection_id(),
        ctx.exchange_id(),
        ctx.protocol(),
    );

    let should_rewrite =
        matches!(ev.event_name_str(), Some("message")) && matches!(ev.data_str(), Some("ping"));
    if !should_rewrite || !ev.can_mutate() {
        return StreamAction::Pass;
    }

    let rewritten = state.rewritten.get() + 1;
    state.rewritten.set(rewritten);

    let _ = ev.set_event_name_str("status");
    let _ = ev.set_id_str(&format!("rewritten-{rewritten}"));
    let _ = ev.set_retry_ms(1500);
    let _ = ev.set_data_str("pong from qcontrol");

    StreamAction::Pass
}

/// Observe stream completion and keep the example state lifecycle explicit.
fn on_sse_close(state: PluginState, ev: &SseCloseEvent) {
    if let Some(state) = state.downcast_ref::<SseState>() {
        let _ = (
            state.connection_id,
            state.exchange_id,
            state.rewritten.get(),
            ev.ctx().connection_id(),
            ev.ctx().exchange_id(),
            ev.ctx().http_version(),
        );
    }
}

export_plugin!(PluginBuilder::new("rust_sse_rewrite")
    .on_sse_open(on_sse_open)
    .on_sse_event(on_sse_event)
    .on_sse_close(on_sse_close));
