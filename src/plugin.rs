//! Plugin builder and export macro
//!
//! Provides a builder pattern for configuring plugins and the `export_plugin!`
//! macro for generating the required C ABI exports.

use crate::error::Error;
use crate::exec::{ExecExitFn, ExecFn, ExecStderrFn, ExecStdinFn, ExecStdoutFn};
use crate::file::{FileCloseFn, FileOpenFn, FileReadFn, FileWriteFn};
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
        // Store the builder in a static for access from wrapper functions
        static PLUGIN_BUILDER: std::sync::OnceLock<$crate::PluginBuilder> =
            std::sync::OnceLock::new();

        fn get_builder() -> &'static $crate::PluginBuilder {
            PLUGIN_BUILDER.get_or_init(|| $builder)
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
                    $crate::file::FileState::empty()
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
        // Static plugin descriptor
        // ====================================================================

        #[no_mangle]
        #[used]
        pub static qcontrol_plugin: $crate::SyncPluginDescriptor =
            $crate::SyncPluginDescriptor($crate::ffi::qcontrol_plugin_t {
                version: $crate::ffi::QCONTROL_PLUGIN_VERSION,
                name: PLUGIN_NAME.as_ptr() as *const std::ffi::c_char,
                // Lifecycle
                on_init: Some(__qcontrol_init_wrapper),
                on_cleanup: Some(__qcontrol_cleanup_wrapper),
                // File
                on_file_open: Some(__qcontrol_file_open_wrapper),
                on_file_read: Some(__qcontrol_file_read_wrapper),
                on_file_write: Some(__qcontrol_file_write_wrapper),
                on_file_close: Some(__qcontrol_file_close_wrapper),
                // Exec
                on_exec: Some(__qcontrol_exec_wrapper),
                on_exec_stdin: Some(__qcontrol_exec_stdin_wrapper),
                on_exec_stdout: Some(__qcontrol_exec_stdout_wrapper),
                on_exec_stderr: Some(__qcontrol_exec_stderr_wrapper),
                on_exec_exit: Some(__qcontrol_exec_exit_wrapper),
                // Net
                on_net_connect: Some(__qcontrol_net_connect_wrapper),
                on_net_accept: Some(__qcontrol_net_accept_wrapper),
                on_net_tls: Some(__qcontrol_net_tls_wrapper),
                on_net_domain: Some(__qcontrol_net_domain_wrapper),
                on_net_protocol: Some(__qcontrol_net_protocol_wrapper),
                on_net_send: Some(__qcontrol_net_send_wrapper),
                on_net_recv: Some(__qcontrol_net_recv_wrapper),
                on_net_close: Some(__qcontrol_net_close_wrapper),
            });
    };
}
