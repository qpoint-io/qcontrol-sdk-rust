//! Plugin builder and export macro
//!
//! Provides a builder pattern for configuring plugins and the `export_plugin!`
//! macro for generating the required C ABI exports.

use crate::exec::{ExecExitFn, ExecFn, ExecStderrFn, ExecStdinFn, ExecStdoutFn};
use crate::ffi::qcontrol_string_view_t;
use crate::file::{FileCloseFn, FileOpFn, FileOpenFn, FileReadFn, FileWriteFn};
use crate::http::{
    HttpExchangeCloseFn, HttpRequestBodyFn, HttpRequestDoneFn, HttpRequestFn,
    HttpRequestTrailersFn, HttpResponseBodyFn, HttpResponseDoneFn, HttpResponseFn,
    HttpResponseTrailersFn,
};
use crate::net::{AcceptFn, CloseFn, ConnectFn, DomainFn, ProtocolFn, RecvFn, SendFn, TlsFn};
use crate::netstat::NetstatFn;
use crate::sse::{SseCloseFn, SseEventFn, SseOpenFn};
use crate::websocket::{WebSocketCloseFn, WebSocketMessageFn, WebSocketOpenFn};

/// Boxed error returned by `on_init` and `on_config` callbacks.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Callback type for `on_init` and `on_config`.
///
/// Receives the config as borrowed UTF-8 JSON (the text `null` when absent
/// at init); the text is valid only during the call, so copy anything
/// retained.
pub type ConfigCallback = fn(&str) -> Result<(), BoxError>;

/// Marks a config as invalid (status 1); its message is the reason reported to
/// the host.
///
/// Return it boxed from `on_init` or `on_config`. Any other error maps to a
/// config-processing failure (status 2) with the error's `to_string()` as the
/// reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidConfig(pub String);

impl std::fmt::Display for InvalidConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InvalidConfig {}

/// Deserialize plugin-owned settings without exposing rejected values in errors.
///
/// Defaults, validation and handling JSON `null` belong to the plugin's chosen
/// type; this helper preserves Serde's decoding rules and returns owned data.
/// An unknown or duplicate field is reported by name, since a field name is
/// never a value; every other failure reports only `invalid config`.
pub fn parse_config<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, InvalidConfig> {
    serde_json::from_str(json).map_err(|error| InvalidConfig(field_error(&error.to_string())))
}

/// Keep Serde's unknown/duplicate field message, which names only fields, and
/// drop its input position. Any other message may quote a value.
fn field_error(message: &str) -> String {
    if message.starts_with("unknown field `") || message.starts_with("duplicate field `") {
        let end = message.rfind(" at line ").unwrap_or(message.len());
        return message[..end].to_string();
    }
    "invalid config".into()
}

/// Run a config callback behind the C boundary shared by `on_init` and
/// `on_config`: decode the borrowed JSON, contain panics, and write bounded
/// error text into the host buffer.
///
/// # Safety
///
/// `config` must be null or point to a valid `qcontrol_string_view_t` whose `data`
/// buffer is readable for `len` bytes for the duration of the call. `error`
/// may be null; otherwise it must be writable for `error_cap` bytes.
#[doc(hidden)]
pub unsafe fn run_config_callback(
    config: *const qcontrol_string_view_t,
    error: *mut std::ffi::c_char,
    error_cap: usize,
    callback: ConfigCallback,
    panic_message: &str,
) -> i32 {
    let write_error = |message: &str| {
        if error_cap == 0 || error.is_null() {
            return;
        }
        let message = message.as_bytes();
        // Reserve one byte for NUL and never touch memory outside the
        // host-provided capacity.
        let len = message.len().min(error_cap - 1);
        // SAFETY: `error` is non-null and `error_cap >= 1` here; `len <= error_cap - 1`
        // so the copy and the NUL at `error.add(len)` stay within `error_cap` bytes.
        // `message` is a Rust-owned string, so it cannot overlap the host buffer.
        unsafe {
            std::ptr::copy_nonoverlapping(message.as_ptr(), error.cast::<u8>(), len);
            *error.add(len) = 0;
        }
    };

    if config.is_null() {
        return 2;
    }

    let config = unsafe { &*config };

    // Borrow exactly the supplied bytes; the host does not provide a terminator.
    let json_bytes = if config.len == 0 {
        &[]
    } else if config.data.is_null() {
        write_error("invalid config JSON");
        return 2;
    } else {
        unsafe { std::slice::from_raw_parts(config.data.cast::<u8>(), config.len) }
    };
    let json = match std::str::from_utf8(json_bytes) {
        Ok(json) => json,
        Err(_) => {
            write_error("invalid config JSON UTF-8");
            return 2;
        }
    };

    // Format and drop the plugin's error inside the boundary too: a plugin
    // error type whose `Display` or `Drop` panics must not unwind into the
    // `extern "C"` wrapper and abort the host.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match callback(json) {
        Ok(()) => crate::ffi::QCONTROL_CONFIG_OK as i32,
        Err(err) => match err.downcast_ref::<InvalidConfig>() {
            Some(invalid) => {
                write_error(&invalid.0);
                crate::ffi::QCONTROL_CONFIG_INVALID as i32
            }
            None => {
                write_error(&err.to_string());
                2
            }
        },
    }));
    match outcome {
        Ok(status) => status,
        Err(payload) => {
            // Leak the payload: its destructor is plugin code that could panic
            // again outside this boundary.
            std::mem::forget(payload);
            write_error(panic_message);
            2
        }
    }
}

/// Plugin builder for configuring qcontrol plugins.
///
/// # Example
///
/// ```rust,ignore
/// use qcontrol::prelude::*;
///
/// fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
///     if ev.path().starts_with("/tmp/secret") {
///         return FileOpenResult::Block;
///     }
///     FileOpenResult::Pass
/// }
///
/// export_plugin!(
///     PluginBuilder::new("my-plugin")
///         .on_file_open(on_open)
/// );
/// ```
pub struct PluginBuilder {
    name: &'static str,
    on_init: Option<ConfigCallback>,
    on_cleanup: Option<fn()>,
    // File callbacks
    on_file_open: Option<FileOpenFn>,
    on_file_read: Option<FileReadFn>,
    on_file_write: Option<FileWriteFn>,
    on_file_close: Option<FileCloseFn>,
    on_file_op: Option<FileOpFn>,
    // Exec callbacks
    on_exec: Option<ExecFn>,
    on_exec_stdin: Option<ExecStdinFn>,
    on_exec_stdout: Option<ExecStdoutFn>,
    on_exec_stderr: Option<ExecStderrFn>,
    on_exec_exit: Option<ExecExitFn>,
    // Net callbacks
    on_net_connect: Option<ConnectFn>,
    on_net_accept: Option<AcceptFn>,
    on_net_tls: Option<TlsFn>,
    on_net_domain: Option<DomainFn>,
    on_net_protocol: Option<ProtocolFn>,
    on_net_send: Option<SendFn>,
    on_net_recv: Option<RecvFn>,
    on_net_close: Option<CloseFn>,
    // HTTP callbacks
    on_http_request: Option<HttpRequestFn>,
    on_http_request_body: Option<HttpRequestBodyFn>,
    on_http_request_trailers: Option<HttpRequestTrailersFn>,
    on_http_request_done: Option<HttpRequestDoneFn>,
    on_http_response: Option<HttpResponseFn>,
    on_http_response_body: Option<HttpResponseBodyFn>,
    on_http_response_trailers: Option<HttpResponseTrailersFn>,
    on_http_response_done: Option<HttpResponseDoneFn>,
    on_http_exchange_close: Option<HttpExchangeCloseFn>,
    // SSE callbacks
    on_sse_open: Option<SseOpenFn>,
    on_sse_event: Option<SseEventFn>,
    on_sse_close: Option<SseCloseFn>,
    // WebSocket callbacks
    on_websocket_open: Option<WebSocketOpenFn>,
    on_websocket_send: Option<WebSocketMessageFn>,
    on_websocket_recv: Option<WebSocketMessageFn>,
    on_websocket_close: Option<WebSocketCloseFn>,
    // Netstat callback
    on_netstat: Option<NetstatFn>,
    // Configuration updates after successful initialization
    on_config: Option<ConfigCallback>,
}

impl PluginBuilder {
    /// Create a new plugin builder with the given name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            on_init: None,
            on_cleanup: None,
            // File
            on_file_open: None,
            on_file_read: None,
            on_file_write: None,
            on_file_close: None,
            on_file_op: None,
            // Exec
            on_exec: None,
            on_exec_stdin: None,
            on_exec_stdout: None,
            on_exec_stderr: None,
            on_exec_exit: None,
            // Net
            on_net_connect: None,
            on_net_accept: None,
            on_net_tls: None,
            on_net_domain: None,
            on_net_protocol: None,
            on_net_send: None,
            on_net_recv: None,
            on_net_close: None,
            // HTTP
            on_http_request: None,
            on_http_request_body: None,
            on_http_request_trailers: None,
            on_http_request_done: None,
            on_http_response: None,
            on_http_response_body: None,
            on_http_response_trailers: None,
            on_http_response_done: None,
            on_http_exchange_close: None,
            // SSE
            on_sse_open: None,
            on_sse_event: None,
            on_sse_close: None,
            // WebSocket
            on_websocket_open: None,
            on_websocket_send: None,
            on_websocket_recv: None,
            on_websocket_close: None,
            on_netstat: None,
            on_config: None,
        }
    }

    /// Set the initialization callback.
    ///
    /// Called once with the initial effective config JSON before the plugin is
    /// published. Return `Err` to abort loading ([`InvalidConfig`] rejects the
    /// config as invalid; any other error is a failure); on failure the host
    /// does not call cleanup, so unwind any partial setup before returning.
    /// Plugins that register no `on_init` accept only a null config.
    /// The borrowed UTF-8 JSON is valid only during this callback; absent
    /// configuration is the text `null`. Copy anything retained.
    pub const fn on_init(mut self, f: ConfigCallback) -> Self {
        self.on_init = Some(f);
        self
    }

    /// Set the cleanup callback.
    ///
    /// Called before the plugin is unloaded.
    pub const fn on_cleanup(mut self, f: fn()) -> Self {
        self.on_cleanup = Some(f);
        self
    }

    /// Set the file open callback.
    ///
    /// Called after open() syscall completes.
    pub const fn on_file_open(mut self, f: FileOpenFn) -> Self {
        self.on_file_open = Some(f);
        self
    }

    /// Set the file read callback.
    ///
    /// Called after read() syscall completes.
    pub const fn on_file_read(mut self, f: FileReadFn) -> Self {
        self.on_file_read = Some(f);
        self
    }

    /// Set the file write callback.
    ///
    /// Called before write() syscall executes.
    pub const fn on_file_write(mut self, f: FileWriteFn) -> Self {
        self.on_file_write = Some(f);
        self
    }

    /// Set the file close callback.
    ///
    /// Called after close() syscall completes.
    pub const fn on_file_close(mut self, f: FileCloseFn) -> Self {
        self.on_file_close = Some(f);
        self
    }

    /// Set the generic file path-operation callback.
    ///
    /// Called for path-oriented filesystem operations outside the stream lifecycle.
    pub const fn on_file_op(mut self, f: FileOpFn) -> Self {
        self.on_file_op = Some(f);
        self
    }

    // ========================================================================
    // Exec callbacks
    // ========================================================================

    /// Set the exec callback.
    ///
    /// Called before exec syscall executes.
    pub const fn on_exec(mut self, f: ExecFn) -> Self {
        self.on_exec = Some(f);
        self
    }

    /// Set the exec stdin callback.
    ///
    /// Called before data is written to child stdin.
    pub const fn on_exec_stdin(mut self, f: ExecStdinFn) -> Self {
        self.on_exec_stdin = Some(f);
        self
    }

    /// Set the exec stdout callback.
    ///
    /// Called after data is read from child stdout.
    pub const fn on_exec_stdout(mut self, f: ExecStdoutFn) -> Self {
        self.on_exec_stdout = Some(f);
        self
    }

    /// Set the exec stderr callback.
    ///
    /// Called after data is read from child stderr.
    pub const fn on_exec_stderr(mut self, f: ExecStderrFn) -> Self {
        self.on_exec_stderr = Some(f);
        self
    }

    /// Set the exec exit callback.
    ///
    /// Called when child process exits.
    pub const fn on_exec_exit(mut self, f: ExecExitFn) -> Self {
        self.on_exec_exit = Some(f);
        self
    }

    // ========================================================================
    // Net callbacks
    // ========================================================================

    /// Set the net connect callback.
    ///
    /// Called after connect() completes.
    pub const fn on_net_connect(mut self, f: ConnectFn) -> Self {
        self.on_net_connect = Some(f);
        self
    }

    /// Set the net accept callback.
    ///
    /// Called after accept() completes.
    pub const fn on_net_accept(mut self, f: AcceptFn) -> Self {
        self.on_net_accept = Some(f);
        self
    }

    /// Set the net TLS callback.
    ///
    /// Called when TLS handshake completes.
    pub const fn on_net_tls(mut self, f: TlsFn) -> Self {
        self.on_net_tls = Some(f);
        self
    }

    /// Set the net domain callback.
    ///
    /// Called when domain name is discovered.
    pub const fn on_net_domain(mut self, f: DomainFn) -> Self {
        self.on_net_domain = Some(f);
        self
    }

    /// Set the net protocol callback.
    ///
    /// Called when application protocol is detected.
    pub const fn on_net_protocol(mut self, f: ProtocolFn) -> Self {
        self.on_net_protocol = Some(f);
        self
    }

    /// Set the net send callback.
    ///
    /// Called before data is sent.
    pub const fn on_net_send(mut self, f: SendFn) -> Self {
        self.on_net_send = Some(f);
        self
    }

    /// Set the net recv callback.
    ///
    /// Called after data is received.
    pub const fn on_net_recv(mut self, f: RecvFn) -> Self {
        self.on_net_recv = Some(f);
        self
    }

    /// Set the net close callback.
    ///
    /// Called when connection is closed.
    pub const fn on_net_close(mut self, f: CloseFn) -> Self {
        self.on_net_close = Some(f);
        self
    }

    // ========================================================================
    // HTTP callbacks
    // ========================================================================

    /// Set the HTTP request callback.
    ///
    /// Called when request headers arrive. Returns an action (Pass/Block/State).
    pub const fn on_http_request(mut self, f: HttpRequestFn) -> Self {
        self.on_http_request = Some(f);
        self
    }

    /// Set the HTTP request body callback.
    ///
    /// Called for each decoded request body chunk.
    pub const fn on_http_request_body(mut self, f: HttpRequestBodyFn) -> Self {
        self.on_http_request_body = Some(f);
        self
    }

    /// Set the HTTP request trailers callback.
    ///
    /// Called when request trailers arrive.
    pub const fn on_http_request_trailers(mut self, f: HttpRequestTrailersFn) -> Self {
        self.on_http_request_trailers = Some(f);
        self
    }

    /// Set the HTTP request done callback.
    ///
    /// Called when the request message is complete.
    pub const fn on_http_request_done(mut self, f: HttpRequestDoneFn) -> Self {
        self.on_http_request_done = Some(f);
        self
    }

    /// Set the HTTP response callback.
    ///
    /// Called when response headers arrive. Returns an action.
    pub const fn on_http_response(mut self, f: HttpResponseFn) -> Self {
        self.on_http_response = Some(f);
        self
    }

    /// Set the HTTP response body callback.
    ///
    /// Called for each decoded response body chunk.
    pub const fn on_http_response_body(mut self, f: HttpResponseBodyFn) -> Self {
        self.on_http_response_body = Some(f);
        self
    }

    /// Set the HTTP response trailers callback.
    ///
    /// Called when response trailers arrive.
    pub const fn on_http_response_trailers(mut self, f: HttpResponseTrailersFn) -> Self {
        self.on_http_response_trailers = Some(f);
        self
    }

    /// Set the HTTP response done callback.
    ///
    /// Called when the response message is complete.
    pub const fn on_http_response_done(mut self, f: HttpResponseDoneFn) -> Self {
        self.on_http_response_done = Some(f);
        self
    }

    /// Set the HTTP exchange close callback.
    ///
    /// Called exactly once per tracked exchange for cleanup.
    pub const fn on_http_exchange_close(mut self, f: HttpExchangeCloseFn) -> Self {
        self.on_http_exchange_close = Some(f);
        self
    }

    // ========================================================================
    // SSE callbacks
    // ========================================================================

    /// Set the SSE open callback.
    ///
    /// Called once when one HTTP response is identified as `text/event-stream`.
    pub const fn on_sse_open(mut self, f: SseOpenFn) -> Self {
        self.on_sse_open = Some(f);
        self
    }

    /// Set the SSE event callback.
    ///
    /// Called once per parsed logical SSE event record.
    pub const fn on_sse_event(mut self, f: SseEventFn) -> Self {
        self.on_sse_event = Some(f);
        self
    }

    /// Set the SSE close callback.
    ///
    /// Called once when the owning SSE response stream ends.
    pub const fn on_sse_close(mut self, f: SseCloseFn) -> Self {
        self.on_sse_close = Some(f);
        self
    }

    // ========================================================================
    // WebSocket callbacks
    // ========================================================================

    /// Set the WebSocket open callback.
    ///
    /// Called once when one HTTP upgrade handshake becomes a WebSocket stream.
    pub const fn on_websocket_open(mut self, f: WebSocketOpenFn) -> Self {
        self.on_websocket_open = Some(f);
        self
    }

    /// Set the WebSocket send callback.
    ///
    /// Called once per logical client-to-server WebSocket message.
    pub const fn on_websocket_send(mut self, f: WebSocketMessageFn) -> Self {
        self.on_websocket_send = Some(f);
        self
    }

    /// Set the WebSocket receive callback.
    ///
    /// Called once per logical server-to-client WebSocket message.
    pub const fn on_websocket_recv(mut self, f: WebSocketMessageFn) -> Self {
        self.on_websocket_recv = Some(f);
        self
    }

    /// Set the WebSocket close callback.
    ///
    /// Called once when the upgraded WebSocket stream ends.
    pub const fn on_websocket_close(mut self, f: WebSocketCloseFn) -> Self {
        self.on_websocket_close = Some(f);
        self
    }

    /// Set the netstat snapshot callback.
    ///
    /// Called with one read-only snapshot. The snapshot is valid only for
    /// the call.
    pub const fn on_netstat(mut self, f: NetstatFn) -> Self {
        self.on_netstat = Some(f);
        self
    }

    /// Set the config update callback.
    ///
    /// Called only for updates to an already initialized plugin, never for the
    /// initial config (use `on_init`). Return `Err(InvalidConfig(..).into())` to
    /// reject the config as invalid; any other error reports a config-processing
    /// failure, as does a contained panic. Validate before mutating active state
    /// so a rejected update preserves the previous config. The borrowed UTF-8
    /// JSON is valid only during this callback; copy anything retained.
    pub const fn on_config(mut self, f: ConfigCallback) -> Self {
        self.on_config = Some(f);
        self
    }

    /// Get the config update callback.
    pub const fn get_on_config(&self) -> Option<ConfigCallback> {
        self.on_config
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Get the plugin name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the init callback.
    pub const fn get_on_init(&self) -> Option<ConfigCallback> {
        self.on_init
    }

    /// Get the cleanup callback.
    pub const fn get_on_cleanup(&self) -> Option<fn()> {
        self.on_cleanup
    }

    /// Get the file open callback.
    pub const fn get_on_file_open(&self) -> Option<FileOpenFn> {
        self.on_file_open
    }

    /// Get the file read callback.
    pub const fn get_on_file_read(&self) -> Option<FileReadFn> {
        self.on_file_read
    }

    /// Get the file write callback.
    pub const fn get_on_file_write(&self) -> Option<FileWriteFn> {
        self.on_file_write
    }

    /// Get the file close callback.
    pub const fn get_on_file_close(&self) -> Option<FileCloseFn> {
        self.on_file_close
    }

    /// Get the file path-operation callback.
    pub const fn get_on_file_op(&self) -> Option<FileOpFn> {
        self.on_file_op
    }

    /// Get the exec callback.
    pub const fn get_on_exec(&self) -> Option<ExecFn> {
        self.on_exec
    }

    /// Get the exec stdin callback.
    pub const fn get_on_exec_stdin(&self) -> Option<ExecStdinFn> {
        self.on_exec_stdin
    }

    /// Get the exec stdout callback.
    pub const fn get_on_exec_stdout(&self) -> Option<ExecStdoutFn> {
        self.on_exec_stdout
    }

    /// Get the exec stderr callback.
    pub const fn get_on_exec_stderr(&self) -> Option<ExecStderrFn> {
        self.on_exec_stderr
    }

    /// Get the exec exit callback.
    pub const fn get_on_exec_exit(&self) -> Option<ExecExitFn> {
        self.on_exec_exit
    }

    /// Get the net connect callback.
    pub const fn get_on_net_connect(&self) -> Option<ConnectFn> {
        self.on_net_connect
    }

    /// Get the net accept callback.
    pub const fn get_on_net_accept(&self) -> Option<AcceptFn> {
        self.on_net_accept
    }

    /// Get the net TLS callback.
    pub const fn get_on_net_tls(&self) -> Option<TlsFn> {
        self.on_net_tls
    }

    /// Get the net domain callback.
    pub const fn get_on_net_domain(&self) -> Option<DomainFn> {
        self.on_net_domain
    }

    /// Get the net protocol callback.
    pub const fn get_on_net_protocol(&self) -> Option<ProtocolFn> {
        self.on_net_protocol
    }

    /// Get the net send callback.
    pub const fn get_on_net_send(&self) -> Option<SendFn> {
        self.on_net_send
    }

    /// Get the net recv callback.
    pub const fn get_on_net_recv(&self) -> Option<RecvFn> {
        self.on_net_recv
    }

    /// Get the net close callback.
    pub const fn get_on_net_close(&self) -> Option<CloseFn> {
        self.on_net_close
    }

    /// Get the HTTP request callback.
    pub const fn get_on_http_request(&self) -> Option<HttpRequestFn> {
        self.on_http_request
    }

    /// Get the HTTP request body callback.
    pub const fn get_on_http_request_body(&self) -> Option<HttpRequestBodyFn> {
        self.on_http_request_body
    }

    /// Get the HTTP request trailers callback.
    pub const fn get_on_http_request_trailers(&self) -> Option<HttpRequestTrailersFn> {
        self.on_http_request_trailers
    }

    /// Get the HTTP request done callback.
    pub const fn get_on_http_request_done(&self) -> Option<HttpRequestDoneFn> {
        self.on_http_request_done
    }

    /// Get the HTTP response callback.
    pub const fn get_on_http_response(&self) -> Option<HttpResponseFn> {
        self.on_http_response
    }

    /// Get the HTTP response body callback.
    pub const fn get_on_http_response_body(&self) -> Option<HttpResponseBodyFn> {
        self.on_http_response_body
    }

    /// Get the HTTP response trailers callback.
    pub const fn get_on_http_response_trailers(&self) -> Option<HttpResponseTrailersFn> {
        self.on_http_response_trailers
    }

    /// Get the HTTP response done callback.
    pub const fn get_on_http_response_done(&self) -> Option<HttpResponseDoneFn> {
        self.on_http_response_done
    }

    /// Get the HTTP exchange close callback.
    pub const fn get_on_http_exchange_close(&self) -> Option<HttpExchangeCloseFn> {
        self.on_http_exchange_close
    }

    /// Get the SSE open callback.
    pub const fn get_on_sse_open(&self) -> Option<SseOpenFn> {
        self.on_sse_open
    }

    /// Get the SSE event callback.
    pub const fn get_on_sse_event(&self) -> Option<SseEventFn> {
        self.on_sse_event
    }

    /// Get the SSE close callback.
    pub const fn get_on_sse_close(&self) -> Option<SseCloseFn> {
        self.on_sse_close
    }

    /// Get the WebSocket open callback.
    pub const fn get_on_websocket_open(&self) -> Option<WebSocketOpenFn> {
        self.on_websocket_open
    }

    /// Get the WebSocket send callback.
    pub const fn get_on_websocket_send(&self) -> Option<WebSocketMessageFn> {
        self.on_websocket_send
    }

    /// Get the WebSocket receive callback.
    pub const fn get_on_websocket_recv(&self) -> Option<WebSocketMessageFn> {
        self.on_websocket_recv
    }

    /// Get the WebSocket close callback.
    pub const fn get_on_websocket_close(&self) -> Option<WebSocketCloseFn> {
        self.on_websocket_close
    }

    /// Get the netstat snapshot callback.
    pub const fn get_on_netstat(&self) -> Option<NetstatFn> {
        self.on_netstat
    }
}

/// Wrapper to make raw plugin descriptor Sync.
///
/// This is safe because the plugin descriptor contains only function pointers
/// and a static string pointer, which are inherently thread-safe.
#[doc(hidden)]
#[repr(transparent)]
pub struct SyncPluginDescriptor(pub crate::ffi::qcontrol_plugin_t);

// SAFETY: The plugin descriptor contains only function pointers and a static
// string pointer. Function pointers are inherently Sync, and static string
// pointers are also Sync.
unsafe impl Sync for SyncPluginDescriptor {}

/// Export a qcontrol plugin descriptor.
///
/// This macro generates the `qcontrol_plugin` symbol that the agent looks for
/// when loading plugins. It creates wrapper functions that convert between
/// Rust types and the C ABI.
///
/// # Example
///
/// ```rust,ignore
/// use qcontrol::prelude::*;
///
/// fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
///     eprintln!("open({})", ev.path());
///     FileOpenResult::Pass
/// }
///
/// export_plugin!(
///     PluginBuilder::new("my-plugin")
///         .on_file_open(on_open)
/// );
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($builder:expr) => {
        // Evaluate the builder at const time so we can inspect which callbacks
        // were registered. This is used to conditionally populate descriptor
        // slots — the runtime treats null vs non-null to decide whether a
        // plugin participates in file/exec/net/http handling.
        const __QCONTROL_BUILDER: $crate::PluginBuilder = $builder;

        // Store the builder in a static for access from wrapper functions
        static PLUGIN_BUILDER: std::sync::OnceLock<$crate::PluginBuilder> =
            std::sync::OnceLock::new();

        fn get_builder() -> &'static $crate::PluginBuilder {
            PLUGIN_BUILDER.get_or_init(|| __QCONTROL_BUILDER)
        }

        // Plugin name as a null-terminated static string
        static PLUGIN_NAME: &[u8] = concat!(module_path!(), "\0").as_bytes();

        // ====================================================================
        // Lifecycle wrappers
        // ====================================================================

        extern "C" fn __qcontrol_init_wrapper(
            config: *const $crate::ffi::qcontrol_string_view_t,
            error: *mut std::ffi::c_char,
            error_cap: usize,
        ) -> i32 {
            let Some(callback) = get_builder().get_on_init() else {
                return 2;
            };
            unsafe {
                $crate::run_config_callback(
                    config,
                    error,
                    error_cap,
                    callback,
                    "init callback panicked",
                )
            }
        }

        extern "C" fn __qcontrol_cleanup_wrapper() {
            let builder = get_builder();
            if let Some(f) = builder.get_on_cleanup() {
                f();
            }
        }

        // ====================================================================
        // Config wrapper
        // ====================================================================

        extern "C" fn __qcontrol_config_wrapper(
            config: *const $crate::ffi::qcontrol_string_view_t,
            error: *mut std::ffi::c_char,
            error_cap: usize,
        ) -> i32 {
            let Some(callback) = get_builder().get_on_config() else {
                return 2;
            };
            unsafe {
                $crate::run_config_callback(
                    config,
                    error,
                    error_cap,
                    callback,
                    "config callback panicked",
                )
            }
        }

        // ====================================================================
        // File wrappers
        // ====================================================================

        extern "C" fn __qcontrol_file_open_wrapper(
            event: *mut $crate::ffi::qcontrol_file_open_event_t,
        ) -> $crate::ffi::qcontrol_file_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_open() {
                let ev = unsafe { $crate::file::FileOpenEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_file_action_t {
                    type_: $crate::ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_file_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_file_read_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_read_event_t,
        ) -> $crate::ffi::qcontrol_file_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_read() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::file::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::file::FileReadEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_file_action_t {
                    type_: $crate::ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_file_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_file_write_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_write_event_t,
        ) -> $crate::ffi::qcontrol_file_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_write() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::file::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::file::FileWriteEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_file_action_t {
                    type_: $crate::ffi::qcontrol_file_action_type_t::QCONTROL_FILE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_file_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_file_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_close_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_file_close() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::file::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::file::FileCloseEvent::from_raw(event) };
                f(file_state, &ev);
            }

            // Clean up SessionState
            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::file::SessionState);
                }
            }
        }

        extern "C" fn __qcontrol_file_op_wrapper(
            event: *mut $crate::ffi::qcontrol_file_op_event_t,
        ) -> $crate::ffi::qcontrol_file_op_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_op() {
                let ev = unsafe { $crate::file::FileOpEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_file_op_action_t {
                    type_: $crate::ffi::qcontrol_file_op_action_type_t_QCONTROL_FILE_OP_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_file_op_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        // ====================================================================
        // Exec wrappers
        // ====================================================================

        extern "C" fn __qcontrol_exec_wrapper(
            event: *mut $crate::ffi::qcontrol_exec_event_t,
        ) -> $crate::ffi::qcontrol_exec_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_exec() {
                let ev = unsafe { $crate::exec::ExecEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_exec_action_t {
                    type_: $crate::ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_exec_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_exec_stdin_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_exec_stdin_event_t,
        ) -> $crate::ffi::qcontrol_exec_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_exec_stdin() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::exec::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::exec::StdinEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_exec_action_t {
                    type_: $crate::ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_exec_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_exec_stdout_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_exec_stdout_event_t,
        ) -> $crate::ffi::qcontrol_exec_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_exec_stdout() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::exec::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::exec::StdoutEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_exec_action_t {
                    type_: $crate::ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_exec_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_exec_stderr_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_exec_stderr_event_t,
        ) -> $crate::ffi::qcontrol_exec_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_exec_stderr() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::exec::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::exec::StderrEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_exec_action_t {
                    type_: $crate::ffi::qcontrol_exec_action_type_t_QCONTROL_EXEC_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_exec_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_exec_exit_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_exec_exit_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_exec_exit() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::exec::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::exec::ExitEvent::from_raw(event) };
                f(file_state, &ev);
            }

            // Clean up SessionState
            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::exec::SessionState);
                }
            }
        }

        // ====================================================================
        // Net wrappers
        // ====================================================================

        extern "C" fn __qcontrol_net_connect_wrapper(
            event: *mut $crate::ffi::qcontrol_net_connect_event_t,
        ) -> $crate::ffi::qcontrol_net_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_connect() {
                let ev = unsafe { $crate::net::ConnectEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_action_t {
                    type_: $crate::ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_accept_wrapper(
            event: *mut $crate::ffi::qcontrol_net_accept_event_t,
        ) -> $crate::ffi::qcontrol_net_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_accept() {
                let ev = unsafe { $crate::net::AcceptEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_action_t {
                    type_: $crate::ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_tls_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_tls_event_t,
        ) -> $crate::ffi::qcontrol_net_phase_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_tls() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::TlsEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_phase_action_t {
                    type_: $crate::ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_phase_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_domain_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_domain_event_t,
        ) -> $crate::ffi::qcontrol_net_phase_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_domain() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::DomainEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_phase_action_t {
                    type_: $crate::ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_phase_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_protocol_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_protocol_event_t,
        ) -> $crate::ffi::qcontrol_net_phase_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_protocol() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::ProtocolEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_phase_action_t {
                    type_: $crate::ffi::qcontrol_net_phase_action_type_t_QCONTROL_NET_PHASE_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_phase_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_send_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_send_event_t,
        ) -> $crate::ffi::qcontrol_net_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_send() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::SendEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_action_t {
                    type_: $crate::ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_recv_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_recv_event_t,
        ) -> $crate::ffi::qcontrol_net_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_net_recv() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::RecvEvent::from_raw(event) };
                let result = f(file_state, &ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_net_action_t {
                    type_: $crate::ffi::qcontrol_net_action_type_t_QCONTROL_NET_ACTION_PASS,
                    __bindgen_anon_1: $crate::ffi::qcontrol_net_action__bindgen_ty_1 {
                        errno_val: 0,
                    },
                }
            }
        }

        extern "C" fn __qcontrol_net_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_close_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_net_close() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let session_state = &*(state as *const $crate::net::SessionState);
                        session_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::net::CloseEvent::from_raw(event) };
                f(file_state, &ev);
            }

            // Clean up SessionState
            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::net::SessionState);
                }
            }
        }

        // ====================================================================
        // HTTP wrappers
        // ====================================================================

        extern "C" fn __qcontrol_http_request_wrapper(
            event: *mut $crate::ffi::qcontrol_http_request_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_request() {
                let mut ev = unsafe { $crate::http::HttpRequestEvent::from_raw(event) };
                let result = f(&mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_request_body_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_body_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_request_body() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let mut ev = unsafe { $crate::http::HttpBodyEvent::from_raw(event) };
                let result = f(file_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_request_trailers_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_trailers_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_request_trailers() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let mut ev = unsafe { $crate::http::HttpTrailersEvent::from_raw(event) };
                let result = f(file_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_request_done_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_message_done_event_t,
        ) {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_request_done() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::http::HttpMessageDoneEvent::from_raw(event) };
                f(file_state, &ev);
            }
        }

        extern "C" fn __qcontrol_http_response_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_response_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_response() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let mut ev = unsafe { $crate::http::HttpResponseEvent::from_raw(event) };
                let result = f(file_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_response_body_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_body_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_response_body() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let mut ev = unsafe { $crate::http::HttpBodyEvent::from_raw(event) };
                let result = f(file_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_response_trailers_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_trailers_event_t,
        ) -> $crate::ffi::qcontrol_http_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_response_trailers() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let mut ev = unsafe { $crate::http::HttpTrailersEvent::from_raw(event) };
                let result = f(file_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_http_action_t {
                    type_: $crate::ffi::qcontrol_http_action_type_t_QCONTROL_HTTP_ACTION_PASS,
                    body_mode:
                        $crate::ffi::qcontrol_http_body_mode_t_QCONTROL_HTTP_BODY_MODE_DEFAULT,
                    __bindgen_anon_1: $crate::ffi::qcontrol_http_action__bindgen_ty_1 {
                        state: std::ptr::null_mut(),
                    },
                }
            }
        }

        extern "C" fn __qcontrol_http_response_done_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_message_done_event_t,
        ) {
            let builder = get_builder();
            if let Some(f) = builder.get_on_http_response_done() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::http::HttpMessageDoneEvent::from_raw(event) };
                f(file_state, &ev);
            }
        }

        extern "C" fn __qcontrol_http_exchange_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_http_exchange_close_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_http_exchange_close() {
                let file_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let http_state = &*(state as *const $crate::http::HttpState);
                        http_state.as_file_state()
                    }
                };
                let ev = unsafe { $crate::http::HttpExchangeCloseEvent::from_raw(event) };
                f(file_state, &ev);
            }

            // Clean up HttpState
            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::http::HttpState);
                }
            }
        }

        // ====================================================================
        // SSE wrappers
        // ====================================================================

        extern "C" fn __qcontrol_sse_open_wrapper(
            event: *mut $crate::ffi::qcontrol_sse_open_event_t,
        ) -> $crate::ffi::qcontrol_stream_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_sse_open() {
                let ev = unsafe { $crate::sse::SseOpenEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_stream_action_t {
                    type_: $crate::ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                    state: std::ptr::null_mut(),
                }
            }
        }

        extern "C" fn __qcontrol_sse_event_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_sse_event_record_t,
        ) -> $crate::ffi::qcontrol_stream_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_sse_event() {
                let plugin_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let stream_state = &*(state as *const $crate::stream::StreamState);
                        stream_state.as_plugin_state()
                    }
                };
                let mut ev = unsafe { $crate::sse::SseEventRecord::from_raw(event) };
                let result = f(plugin_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_stream_action_t {
                    type_: $crate::ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                    state: std::ptr::null_mut(),
                }
            }
        }

        extern "C" fn __qcontrol_sse_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_sse_close_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_sse_close() {
                let plugin_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let stream_state = &*(state as *const $crate::stream::StreamState);
                        stream_state.as_plugin_state()
                    }
                };
                let ev = unsafe { $crate::sse::SseCloseEvent::from_raw(event) };
                f(plugin_state, &ev);
            }

            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::stream::StreamState);
                }
            }
        }

        // ====================================================================
        // WebSocket wrappers
        // ====================================================================

        extern "C" fn __qcontrol_websocket_open_wrapper(
            event: *mut $crate::ffi::qcontrol_websocket_open_event_t,
        ) -> $crate::ffi::qcontrol_stream_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_websocket_open() {
                let ev = unsafe { $crate::websocket::WebSocketOpenEvent::from_raw(event) };
                let result = f(&ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_stream_action_t {
                    type_: $crate::ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                    state: std::ptr::null_mut(),
                }
            }
        }

        extern "C" fn __qcontrol_websocket_send_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_websocket_message_event_t,
        ) -> $crate::ffi::qcontrol_stream_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_websocket_send() {
                let plugin_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let stream_state = &*(state as *const $crate::stream::StreamState);
                        stream_state.as_plugin_state()
                    }
                };
                let mut ev = unsafe { $crate::websocket::WebSocketMessageEvent::from_raw(event) };
                let result = f(plugin_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_stream_action_t {
                    type_: $crate::ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                    state: std::ptr::null_mut(),
                }
            }
        }

        extern "C" fn __qcontrol_websocket_recv_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_websocket_message_event_t,
        ) -> $crate::ffi::qcontrol_stream_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_websocket_recv() {
                let plugin_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let stream_state = &*(state as *const $crate::stream::StreamState);
                        stream_state.as_plugin_state()
                    }
                };
                let mut ev = unsafe { $crate::websocket::WebSocketMessageEvent::from_raw(event) };
                let result = f(plugin_state, &mut ev);
                result.to_ffi()
            } else {
                $crate::ffi::qcontrol_stream_action_t {
                    type_: $crate::ffi::qcontrol_stream_action_type_t_QCONTROL_STREAM_ACTION_PASS,
                    state: std::ptr::null_mut(),
                }
            }
        }

        extern "C" fn __qcontrol_websocket_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_websocket_close_event_t,
        ) {
            let builder = get_builder();

            if let Some(f) = builder.get_on_websocket_close() {
                let plugin_state = if state.is_null() {
                    $crate::PluginState::empty()
                } else {
                    unsafe {
                        let stream_state = &*(state as *const $crate::stream::StreamState);
                        stream_state.as_plugin_state()
                    }
                };
                let ev = unsafe { $crate::websocket::WebSocketCloseEvent::from_raw(event) };
                f(plugin_state, &ev);
            }

            if !state.is_null() {
                unsafe {
                    let _ = Box::from_raw(state as *mut $crate::stream::StreamState);
                }
            }
        }

        extern "C" fn __qcontrol_netstat_wrapper(
            snapshot: *const $crate::ffi::qcontrol_netstat_snapshot_t,
        ) {
            let builder = get_builder();
            if let Some(f) = builder.get_on_netstat() {
                if snapshot.is_null() {
                    return;
                }
                // The host keeps the snapshot alive until this wrapper returns.
                let ev = unsafe { $crate::netstat::NetstatSnapshot::from_raw(snapshot) };
                f(&ev);
            }
        }

        // ====================================================================
        // Static plugin descriptor
        // ====================================================================

        // Only populate descriptor slots for callbacks the user actually
        // registered. The runtime checks nullability to decide whether a
        // plugin participates in file/exec/net/http handling — exporting
        // a non-null wrapper for an unregistered callback would cause the
        // runtime to enable unnecessary tracking/streaming for this plugin.
        #[no_mangle]
        #[used]
        pub static qcontrol_plugin: $crate::SyncPluginDescriptor =
            $crate::SyncPluginDescriptor($crate::ffi::qcontrol_plugin_t {
                version: $crate::ffi::QCONTROL_PLUGIN_VERSION,
                name: PLUGIN_NAME.as_ptr() as *const std::ffi::c_char,
                // Lifecycle (init only when registered; null accepts only a
                // null initial config, so configless plugins stay operational)
                on_init: if __QCONTROL_BUILDER.get_on_init().is_some() {
                    Some(__qcontrol_init_wrapper)
                } else {
                    None
                },
                on_cleanup: Some(__qcontrol_cleanup_wrapper),
                // File
                on_file_open: if __QCONTROL_BUILDER.get_on_file_open().is_some() {
                    Some(__qcontrol_file_open_wrapper)
                } else {
                    None
                },
                on_file_read: if __QCONTROL_BUILDER.get_on_file_read().is_some() {
                    Some(__qcontrol_file_read_wrapper)
                } else {
                    None
                },
                on_file_write: if __QCONTROL_BUILDER.get_on_file_write().is_some() {
                    Some(__qcontrol_file_write_wrapper)
                } else {
                    None
                },
                on_file_close: if __QCONTROL_BUILDER.get_on_file_close().is_some() {
                    Some(__qcontrol_file_close_wrapper)
                } else {
                    None
                },
                on_file_op: if __QCONTROL_BUILDER.get_on_file_op().is_some() {
                    Some(__qcontrol_file_op_wrapper)
                } else {
                    None
                },
                // Exec
                on_exec: if __QCONTROL_BUILDER.get_on_exec().is_some() {
                    Some(__qcontrol_exec_wrapper)
                } else {
                    None
                },
                on_exec_stdin: if __QCONTROL_BUILDER.get_on_exec_stdin().is_some() {
                    Some(__qcontrol_exec_stdin_wrapper)
                } else {
                    None
                },
                on_exec_stdout: if __QCONTROL_BUILDER.get_on_exec_stdout().is_some() {
                    Some(__qcontrol_exec_stdout_wrapper)
                } else {
                    None
                },
                on_exec_stderr: if __QCONTROL_BUILDER.get_on_exec_stderr().is_some() {
                    Some(__qcontrol_exec_stderr_wrapper)
                } else {
                    None
                },
                on_exec_exit: if __QCONTROL_BUILDER.get_on_exec_exit().is_some() {
                    Some(__qcontrol_exec_exit_wrapper)
                } else {
                    None
                },
                // Net
                on_net_connect: if __QCONTROL_BUILDER.get_on_net_connect().is_some() {
                    Some(__qcontrol_net_connect_wrapper)
                } else {
                    None
                },
                on_net_accept: if __QCONTROL_BUILDER.get_on_net_accept().is_some() {
                    Some(__qcontrol_net_accept_wrapper)
                } else {
                    None
                },
                on_net_tls: if __QCONTROL_BUILDER.get_on_net_tls().is_some() {
                    Some(__qcontrol_net_tls_wrapper)
                } else {
                    None
                },
                on_net_domain: if __QCONTROL_BUILDER.get_on_net_domain().is_some() {
                    Some(__qcontrol_net_domain_wrapper)
                } else {
                    None
                },
                on_net_protocol: if __QCONTROL_BUILDER.get_on_net_protocol().is_some() {
                    Some(__qcontrol_net_protocol_wrapper)
                } else {
                    None
                },
                on_net_send: if __QCONTROL_BUILDER.get_on_net_send().is_some() {
                    Some(__qcontrol_net_send_wrapper)
                } else {
                    None
                },
                on_net_recv: if __QCONTROL_BUILDER.get_on_net_recv().is_some() {
                    Some(__qcontrol_net_recv_wrapper)
                } else {
                    None
                },
                on_net_close: if __QCONTROL_BUILDER.get_on_net_close().is_some() {
                    Some(__qcontrol_net_close_wrapper)
                } else {
                    None
                },
                // HTTP
                on_http_request: if __QCONTROL_BUILDER.get_on_http_request().is_some() {
                    Some(__qcontrol_http_request_wrapper)
                } else {
                    None
                },
                on_http_request_body: if __QCONTROL_BUILDER.get_on_http_request_body().is_some() {
                    Some(__qcontrol_http_request_body_wrapper)
                } else {
                    None
                },
                on_http_request_trailers: if __QCONTROL_BUILDER
                    .get_on_http_request_trailers()
                    .is_some()
                {
                    Some(__qcontrol_http_request_trailers_wrapper)
                } else {
                    None
                },
                on_http_request_done: if __QCONTROL_BUILDER.get_on_http_request_done().is_some() {
                    Some(__qcontrol_http_request_done_wrapper)
                } else {
                    None
                },
                on_http_response: if __QCONTROL_BUILDER.get_on_http_response().is_some() {
                    Some(__qcontrol_http_response_wrapper)
                } else {
                    None
                },
                on_http_response_body: if __QCONTROL_BUILDER.get_on_http_response_body().is_some() {
                    Some(__qcontrol_http_response_body_wrapper)
                } else {
                    None
                },
                on_http_response_trailers: if __QCONTROL_BUILDER
                    .get_on_http_response_trailers()
                    .is_some()
                {
                    Some(__qcontrol_http_response_trailers_wrapper)
                } else {
                    None
                },
                on_http_response_done: if __QCONTROL_BUILDER.get_on_http_response_done().is_some() {
                    Some(__qcontrol_http_response_done_wrapper)
                } else {
                    None
                },
                on_http_exchange_close: if __QCONTROL_BUILDER.get_on_http_exchange_close().is_some()
                {
                    Some(__qcontrol_http_exchange_close_wrapper)
                } else {
                    None
                },
                // SSE
                on_sse_open: if __QCONTROL_BUILDER.get_on_sse_open().is_some() {
                    Some(__qcontrol_sse_open_wrapper)
                } else {
                    None
                },
                on_sse_event: if __QCONTROL_BUILDER.get_on_sse_event().is_some() {
                    Some(__qcontrol_sse_event_wrapper)
                } else {
                    None
                },
                on_sse_close: if __QCONTROL_BUILDER.get_on_sse_close().is_some() {
                    Some(__qcontrol_sse_close_wrapper)
                } else {
                    None
                },
                // WebSocket
                on_websocket_open: if __QCONTROL_BUILDER.get_on_websocket_open().is_some() {
                    Some(__qcontrol_websocket_open_wrapper)
                } else {
                    None
                },
                on_websocket_send: if __QCONTROL_BUILDER.get_on_websocket_send().is_some() {
                    Some(__qcontrol_websocket_send_wrapper)
                } else {
                    None
                },
                on_websocket_recv: if __QCONTROL_BUILDER.get_on_websocket_recv().is_some() {
                    Some(__qcontrol_websocket_recv_wrapper)
                } else {
                    None
                },
                on_websocket_close: if __QCONTROL_BUILDER.get_on_websocket_close().is_some() {
                    Some(__qcontrol_websocket_close_wrapper)
                } else {
                    None
                },
                on_netstat: if __QCONTROL_BUILDER.get_on_netstat().is_some() {
                    Some(__qcontrol_netstat_wrapper)
                } else {
                    None
                },
                on_config: if __QCONTROL_BUILDER.get_on_config().is_some() {
                    Some(__qcontrol_config_wrapper)
                } else {
                    None
                },
            });
    };
}

#[cfg(test)]
mod config_tests {
    //! Exercises the exported init and update C callback boundaries, including
    //! borrowed data and bounded error-buffer behavior.

    use super::{InvalidConfig, PluginBuilder};
    use crate::ffi::qcontrol_string_view_t;
    use crate::BoxError;
    use std::ffi::c_char;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    static INIT_JSON: Mutex<Vec<u8>> = Mutex::new(Vec::new());
    static UPDATE_JSON: Mutex<Vec<u8>> = Mutex::new(Vec::new());
    static RECEIVED_POINTER: AtomicUsize = AtomicUsize::new(0);

    /// Ordinary plugin error, distinct from rejected configuration.
    #[derive(Debug)]
    struct Internal;

    impl std::fmt::Display for Internal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("internal failure")
        }
    }

    impl std::error::Error for Internal {}

    /// Error whose formatting panics, as plugin-defined error types may.
    #[derive(Debug)]
    struct PanicsOnDisplay;

    impl std::fmt::Display for PanicsOnDisplay {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("display panic")
        }
    }

    impl std::error::Error for PanicsOnDisplay {}

    /// Error whose destructor panics after its message has been formatted.
    #[derive(Debug)]
    struct PanicsOnDrop;

    impl std::fmt::Display for PanicsOnDrop {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("drop failure")
        }
    }

    impl std::error::Error for PanicsOnDrop {}

    impl Drop for PanicsOnDrop {
        fn drop(&mut self) {
            panic!("drop panic")
        }
    }

    /// Return representative plugin failures through the real ABI wrappers.
    fn respond(json: &str) -> Result<(), BoxError> {
        RECEIVED_POINTER.store(json.as_ptr() as usize, Ordering::Relaxed);
        match json {
            "invalid" => Err(InvalidConfig("bad field".into()).into()),
            "failed" => Err(Internal.into()),
            "panic" => panic!("sentinel panic"),
            "payload-drop-panics" => std::panic::panic_any(PanicsOnDrop),
            "display-panics" => Err(PanicsOnDisplay.into()),
            "drop-panics" => Err(PanicsOnDrop.into()),
            _ => Ok(()),
        }
    }

    /// Record initial input before exercising result translation.
    fn init(json: &str) -> Result<(), BoxError> {
        *INIT_JSON.lock().unwrap() = json.as_bytes().to_vec();
        respond(json)
    }

    /// Record update input independently from initialization.
    fn update(json: &str) -> Result<(), BoxError> {
        *UPDATE_JSON.lock().unwrap() = json.as_bytes().to_vec();
        respond(json)
    }

    crate::export_plugin!(PluginBuilder::new("config-tests")
        .on_init(init)
        .on_config(update));

    type ConfigFn = unsafe extern "C" fn(*const qcontrol_string_view_t, *mut c_char, usize) -> i32;

    /// Pass a borrowed byte view and caller-sized diagnostic storage to the ABI.
    fn invoke(callback: ConfigFn, json: &[u8], error: &mut [u8], error_cap: usize) -> i32 {
        let config = qcontrol_string_view_t {
            data: json.as_ptr().cast::<c_char>(),
            len: json.len(),
        };
        unsafe { callback(&config, error.as_mut_ptr().cast::<c_char>(), error_cap) }
    }

    /// Exercise both descriptor callbacks with the same boundary cases.
    fn callbacks() -> [(&'static str, ConfigFn, &'static Mutex<Vec<u8>>); 2] {
        [
            ("init", qcontrol_plugin.0.on_init.unwrap(), &INIT_JSON),
            ("update", qcontrol_plugin.0.on_config.unwrap(), &UPDATE_JSON),
        ]
    }

    /// Protect borrowed input, failure classification and bounded diagnostics.
    #[test]
    fn init_and_update_callbacks_map_results_and_bound_error_writes() {
        for (name, callback, received) in callbacks() {
            let mut error = [b'X'; 64];
            let cap = error.len();
            let json = "{\"snow\":\"☃\"}".as_bytes();
            // Invalid trailing bytes make scanning beyond the supplied length
            // observable; the callback must borrow the original allocation.
            let mut storage = json.to_vec();
            storage.push(0xff);
            let input = &storage[..json.len()];
            assert_eq!(invoke(callback, input, &mut error, cap), 0, "{name}");
            assert_eq!(received.lock().unwrap().as_slice(), json, "{name}");
            assert_eq!(
                RECEIVED_POINTER.load(Ordering::Relaxed),
                input.as_ptr() as usize,
                "{name}"
            );

            assert_eq!(invoke(callback, b"null", &mut error, cap), 0, "{name}");
            assert_eq!(received.lock().unwrap().as_slice(), b"null", "{name}");

            error.fill(b'X');
            assert_eq!(invoke(callback, b"invalid", &mut error, cap), 1, "{name}");
            assert_eq!(&error[..10], b"bad field\0", "{name}");

            error.fill(b'X');
            assert_eq!(invoke(callback, b"failed", &mut error, cap), 2, "{name}");
            assert_eq!(&error[..17], b"internal failure\0", "{name}");

            let mut guarded = [b'G'; 7];
            assert_eq!(
                invoke(callback, b"invalid", &mut guarded[2..5], 3),
                1,
                "{name}"
            );
            assert_eq!(guarded, *b"GGba\0GG", "{name}");

            guarded.fill(b'G');
            assert_eq!(
                invoke(callback, b"invalid", &mut guarded[2..3], 1),
                1,
                "{name}"
            );
            assert_eq!(guarded, *b"GG\0GGGG", "{name}");

            let mut zero_capacity = [b'Z'; 4];
            assert_eq!(
                invoke(callback, b"invalid", &mut zero_capacity, 0),
                1,
                "{name}"
            );
            assert_eq!(zero_capacity, *b"ZZZZ", "{name}");

            // A null output pointer suppresses diagnostics at any capacity.
            for json in [&b"invalid"[..], b"failed", b"panic"] {
                let config = qcontrol_string_view_t {
                    data: json.as_ptr().cast(),
                    len: json.len(),
                };
                for capacity in [0, 64] {
                    let status = unsafe { callback(&config, std::ptr::null_mut(), capacity) };
                    assert_eq!(status, if json == b"invalid" { 1 } else { 2 }, "{name}");
                }
            }

            received.lock().unwrap().clear();
            assert_eq!(invoke(callback, &[0xff], &mut error, cap), 2, "{name}");
            assert!(error.contains(&0), "{name}");
            assert!(received.lock().unwrap().is_empty(), "{name}");

            // Invalid views fail without invoking plugin code; a zero-length
            // view may have a null data pointer and is passed through as "".
            let mut view = qcontrol_string_view_t {
                data: std::ptr::null(),
                len: 1,
            };
            error.fill(b'X');
            assert_eq!(
                unsafe { callback(&view, error.as_mut_ptr().cast(), cap) },
                2,
                "{name}"
            );
            assert!(error.contains(&0), "{name}");
            assert!(received.lock().unwrap().is_empty(), "{name}");
            view.len = 0;
            assert_eq!(
                unsafe { callback(&view, std::ptr::null_mut(), 0) },
                0,
                "{name}"
            );
            assert_eq!(
                unsafe { callback(std::ptr::null(), std::ptr::null_mut(), 0) },
                2,
                "{name}"
            );

            error.fill(b'X');
            assert_eq!(invoke(callback, b"panic", &mut error, cap), 2, "{name}");
            assert!(error.contains(&0), "{name}");

            // A plugin error that panics while being formatted or dropped
            // is contained as a failure instead of aborting the host.
            for json in [
                &b"display-panics"[..],
                b"drop-panics",
                b"payload-drop-panics",
            ] {
                error.fill(b'X');
                assert_eq!(invoke(callback, json, &mut error, cap), 2, "{name}");
                assert!(error.contains(&0), "{name}");
            }
        }
    }
}
