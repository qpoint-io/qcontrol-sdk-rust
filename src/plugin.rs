//! Plugin builder and export macro
//!
//! Provides a builder pattern for configuring plugins and the `export_plugin!`
//! macro for generating the required C ABI exports.

use crate::error::Error;
use crate::exec::{ExecExitFn, ExecFn, ExecStderrFn, ExecStdinFn, ExecStdoutFn};
use crate::file::{FileCloseFn, FileOpenFn, FileReadFn, FileWriteFn};
use crate::http::{
    HttpExchangeCloseFn, HttpRequestBodyFn, HttpRequestDoneFn, HttpRequestFn,
    HttpRequestTrailersFn, HttpResponseBodyFn, HttpResponseDoneFn, HttpResponseFn,
    HttpResponseTrailersFn,
};
use crate::net::{AcceptFn, CloseFn, ConnectFn, DomainFn, ProtocolFn, RecvFn, SendFn, TlsFn};

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
    on_init: Option<fn() -> Result<(), Error>>,
    on_cleanup: Option<fn()>,
    // File callbacks
    on_file_open: Option<FileOpenFn>,
    on_file_read: Option<FileReadFn>,
    on_file_write: Option<FileWriteFn>,
    on_file_close: Option<FileCloseFn>,
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
        }
    }

    /// Set the initialization callback.
    ///
    /// Called after the plugin is loaded. Return `Err` to abort loading.
    pub const fn on_init(mut self, f: fn() -> Result<(), Error>) -> Self {
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
    // Getters
    // ========================================================================

    /// Get the plugin name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the init callback.
    pub const fn get_on_init(&self) -> Option<fn() -> Result<(), Error>> {
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

        extern "C" fn __qcontrol_init_wrapper() -> i32 {
            let builder = get_builder();
            if let Some(f) = builder.get_on_init() {
                match f() {
                    Ok(()) => 0,
                    Err(_) => -1,
                }
            } else {
                0
            }
        }

        extern "C" fn __qcontrol_cleanup_wrapper() {
            let builder = get_builder();
            if let Some(f) = builder.get_on_cleanup() {
                f();
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
        ) {
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
                f(file_state, &ev);
            }
        }

        extern "C" fn __qcontrol_net_domain_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_domain_event_t,
        ) {
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
                f(file_state, &ev);
            }
        }

        extern "C" fn __qcontrol_net_protocol_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_net_protocol_event_t,
        ) {
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
                f(file_state, &ev);
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
                // Lifecycle (always present — init/cleanup are cheap no-ops)
                on_init: Some(__qcontrol_init_wrapper),
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
            });
    };
}
