//! HTTP rewrite plugin example.
//!
//! Demonstrates the coalesced HTTP Rust SDK surface:
//! - request and response header mutation through the existing callbacks
//! - buffered JSON body replacement through the existing body callback
//! - per-exchange state carried from request through response

use std::cell::Cell;

use qcontrol::prelude::*;
use serde_json::Value;

/// Per-exchange state that decides whether the response body should be
/// rewritten as JSON.
struct ExchangeState {
    rewrite_response: Cell<bool>,
}

/// Decide whether this exchange should be rewritten and normalize request
/// headers before the body callbacks begin.
fn on_http_request(ev: &mut HttpRequestEvent) -> HttpRequestAction {
    if let Some(mut headers) = ev.headers_mut() {
        headers.remove_str("proxy-connection");
        headers.set_str("x-qcontrol", "1");
    }

    let rewrite_response = ev.path() == b"/api/profile";
    HttpRequestAction::State(Box::new(ExchangeState {
        rewrite_response: Cell::new(rewrite_response),
    }))
}

/// Normalize the response headers and request buffered-body scheduling when
/// this exchange will replace the JSON response body.
fn on_http_response(state: PluginState, ev: &mut HttpResponseEvent) -> HttpAction {
    let Some(state) = state.downcast_ref::<ExchangeState>() else {
        return HttpAction::Pass;
    };

    if !state.rewrite_response.get() {
        return HttpAction::Pass;
    }

    if let Some(mut headers) = ev.headers_mut() {
        headers.set_str("content-type", "application/json");
    }

    HttpAction::Pass.with_body_mode(HttpBodyMode::Buffer)
}

/// Replace the full JSON response body when the host provides buffered-body
/// handling for this message.
fn on_http_response_body(state: PluginState, ev: &mut HttpBodyEvent) -> HttpAction {
    let Some(state) = state.downcast_ref::<ExchangeState>() else {
        return HttpAction::Pass;
    };

    if !state.rewrite_response.get() || !ev.end_of_stream() {
        return HttpAction::Pass;
    }

    let Ok(mut value) = ev.body_json::<Value>() else {
        return HttpAction::Pass;
    };

    if let Some(object) = value.as_object_mut() {
        object.insert("rewritten".into(), Value::Bool(true));
    }

    let _ = ev.set_body_json(&value);
    HttpAction::Pass
}

export_plugin!(
    PluginBuilder::new("rust_http_rewrite")
        .on_http_request(on_http_request)
        .on_http_response(on_http_response)
        .on_http_response_body(on_http_response_body)
);
