//! HTTP logger plugin - logs all HTTP exchange events to stderr.
//!
//! Mirrors the Zig http-logger plugin. Exercises all 9 HTTP callbacks
//! and tracks per-exchange state (method, target, body sizes, status code).

use std::cell::Cell;

use qcontrol::prelude::*;

/// Per-exchange state tracked from request through close.
///
/// Uses `Cell` for fields mutated by later callbacks since `FileState`
/// only provides shared references.
struct ExchangeState {
    exchange_id: u64,
    method: String,
    raw_target: String,
    status_code: Cell<u16>,
    request_body_bytes: Cell<u64>,
    response_body_bytes: Cell<u64>,
}

fn on_http_request(ev: &HttpRequestEvent) -> HttpRequestAction {
    let ctx = ev.ctx();
    let exchange_id = ctx.exchange_id();
    let method = String::from_utf8_lossy(ev.method()).into_owned();
    let raw_target = String::from_utf8_lossy(ev.raw_target()).into_owned();

    eprintln!(
        "[http_logger.rs] request exchange={} {} {}",
        exchange_id, method, raw_target,
    );
    eprintln!(
        "[http_logger.rs]   version={:?} stream_id={:?} headers={}",
        ctx.version(),
        ctx.stream_id(),
        ev.header_count(),
    );

    let state = ExchangeState {
        exchange_id,
        method,
        raw_target,
        status_code: Cell::new(0),
        request_body_bytes: Cell::new(0),
        response_body_bytes: Cell::new(0),
    };

    HttpRequestAction::State(Box::new(state))
}

fn on_http_request_body(state: FileState, ev: &HttpBodyEvent) -> HttpAction {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        s.request_body_bytes
            .set(s.request_body_bytes.get() + ev.bytes().len() as u64);
        eprintln!(
            "[http_logger.rs] request_body exchange={} bytes={} offset={}",
            s.exchange_id,
            ev.bytes().len(),
            ev.offset(),
        );
    }
    HttpAction::Pass
}

fn on_http_request_trailers(state: FileState, ev: &HttpTrailersEvent) -> HttpAction {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        eprintln!(
            "[http_logger.rs] request_trailers exchange={} count={}",
            s.exchange_id,
            ev.header_count(),
        );
    }
    HttpAction::Pass
}

fn on_http_request_done(state: FileState, ev: &HttpMessageDoneEvent) {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        eprintln!(
            "[http_logger.rs] request_done exchange={} body_bytes={}",
            s.exchange_id,
            ev.body_bytes(),
        );
    }
}

fn on_http_response(state: FileState, ev: &HttpResponseEvent) -> HttpAction {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        s.status_code.set(ev.status_code());
        eprintln!(
            "[http_logger.rs] response exchange={} status={} {} {}",
            s.exchange_id,
            ev.status_code(),
            s.method,
            s.raw_target,
        );
        eprintln!(
            "[http_logger.rs]   version={:?} headers={}",
            ev.ctx().version(),
            ev.header_count(),
        );
    }
    HttpAction::Pass
}

fn on_http_response_body(state: FileState, ev: &HttpBodyEvent) -> HttpAction {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        s.response_body_bytes
            .set(s.response_body_bytes.get() + ev.bytes().len() as u64);
        eprintln!(
            "[http_logger.rs] response_body exchange={} bytes={} offset={}",
            s.exchange_id,
            ev.bytes().len(),
            ev.offset(),
        );
    }
    HttpAction::Pass
}

fn on_http_response_trailers(state: FileState, ev: &HttpTrailersEvent) -> HttpAction {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        eprintln!(
            "[http_logger.rs] response_trailers exchange={} count={}",
            s.exchange_id,
            ev.header_count(),
        );
    }
    HttpAction::Pass
}

fn on_http_response_done(state: FileState, ev: &HttpMessageDoneEvent) {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        eprintln!(
            "[http_logger.rs] response_done exchange={} body_bytes={}",
            s.exchange_id,
            ev.body_bytes(),
        );
    }
}

fn on_http_exchange_close(state: FileState, ev: &HttpExchangeCloseEvent) {
    if let Some(s) = state.downcast_ref::<ExchangeState>() {
        eprintln!(
            "[http_logger.rs] close exchange={} status={} request_bytes={} response_bytes={} reason={:?}",
            s.exchange_id,
            s.status_code.get(),
            s.request_body_bytes.get(),
            s.response_body_bytes.get(),
            ev.reason(),
        );
        let flags = ev.flags();
        eprintln!(
            "[http_logger.rs]   request_done={} response_done={}",
            flags.request_done(),
            flags.response_done(),
        );
    }
}

export_plugin!(
    PluginBuilder::new("rust_http_logger")
        .on_http_request(on_http_request)
        .on_http_request_body(on_http_request_body)
        .on_http_request_trailers(on_http_request_trailers)
        .on_http_request_done(on_http_request_done)
        .on_http_response(on_http_response)
        .on_http_response_body(on_http_response_body)
        .on_http_response_trailers(on_http_response_trailers)
        .on_http_response_done(on_http_response_done)
        .on_http_exchange_close(on_http_exchange_close)
);
