//! qcontrol SDK for Rust
//!
//! This crate provides idiomatic Rust bindings for the qcontrol plugin SDK,
//! enabling you to write file, exec, and network operation filters in safe Rust.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use qcontrol::prelude::*;
//!
//! fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
//!     if ev.path().starts_with("/tmp/secret") {
//!         return FileOpenResult::Block;
//!     }
//!     FileOpenResult::Pass
//! }
//!
//! export_plugin!(
//!     PluginBuilder::new("my-plugin")
//!         .on_file_open(on_open)
//! );
//! ```
//!
//! # Plugin Model
//!
//! The SDK uses a descriptor-based model where plugins export a single
//! `qcontrol_plugin` symbol containing all callbacks:
//!
//! ## Lifecycle
//! - **on_init** - Called once with the initial effective config JSON before
//!   the plugin is published (optional)
//! - **on_config** - Called only for config updates after initialization
//! - **on_cleanup** - Called before plugin unload (optional)
//!
//! ## File Operations
//! - **on_file_open** - Called after open() syscall
//! - **on_file_read** - Called after read() syscall
//! - **on_file_write** - Called before write() syscall
//! - **on_file_close** - Called after close() syscall
//!
//! ## Exec Operations (v1 spec)
//! - **on_exec** - Called before exec syscall
//! - **on_exec_stdin** - Called before data is written to child stdin
//! - **on_exec_stdout** - Called after data is read from child stdout
//! - **on_exec_stderr** - Called after data is read from child stderr
//! - **on_exec_exit** - Called when child process exits
//!
//! ## Network Operations (v1 spec)
//! - **on_net_connect** - Called after connect() completes
//! - **on_net_accept** - Called after accept() completes
//! - **on_net_tls** - Called when TLS handshake completes
//! - **on_net_domain** - Called when domain name is discovered
//! - **on_net_protocol** - Called when protocol is detected
//! - **on_net_send** - Called before data is sent
//! - **on_net_recv** - Called after data is received
//! - **on_net_close** - Called when connection is closed
//!
//! ## Netstat
//! - **on_netstat** - Called with batches of typed socket events
//!
//! # Sessions and State
//!
//! When a file is opened, you can return a `FileSession` to configure
//! transforms and track state across operations:
//!
//! ```rust,ignore
//! use qcontrol::prelude::*;
//!
//! struct MyState { bytes_read: usize }
//!
//! fn on_open(ev: &FileOpenEvent) -> FileOpenResult {
//!     if ev.path().ends_with(".log") {
//!         return FileOpenResult::Session(
//!             FileSession::builder()
//!                 .state(MyState { bytes_read: 0 })
//!                 .read(FileRwConfig::new().prefix_str("[LOG] "))
//!                 .build()
//!         );
//!     }
//!     FileOpenResult::Pass
//! }
//!
//! fn on_close(state: PluginState, _ev: &FileCloseEvent) {
//!     if let Some(s) = state.downcast_ref::<MyState>() {
//!         eprintln!("Total bytes read: {}", s.bytes_read);
//!     }
//! }
//!
//! export_plugin!(
//!     PluginBuilder::new("stateful-plugin")
//!         .on_file_open(on_open)
//!         .on_file_close(on_close)
//! );
//! ```

// Internal modules
pub mod buffer;
mod error;
pub mod exec;
#[doc(hidden)]
pub mod ffi;
pub mod file;
pub mod http;
mod logger;
pub mod net;
pub mod netstat;
mod plugin;
pub mod process;
pub mod sse;
mod state;
pub mod stream;
pub mod websocket;

// Re-export public API
pub use buffer::{Buffer, BufferRef};
pub use error::Error;
pub use logger::Logger;
#[doc(hidden)]
pub use plugin::run_config_callback;
pub use plugin::{
    parse_config, BoxError, ConfigCallback, InvalidConfig, PluginBuilder, SyncPluginDescriptor,
};
pub use process::{ProcessIdentity, ProcessLineageEntry};
pub use sse::{SseCloseEvent, SseCloseFn, SseEventFn, SseEventRecord, SseOpenEvent, SseOpenFn};
pub use state::{FileState, PluginState};
pub use stream::{StreamAction, StreamContext, StreamDirection, StreamProtocol};
pub use websocket::{
    WebSocketCloseEvent, WebSocketCloseFn, WebSocketMessageEvent, WebSocketMessageFn,
    WebSocketMessageKind, WebSocketOpenEvent, WebSocketOpenFn,
};

// Re-export file module types at top level for convenience
pub use file::{
    FileAction, FileCloseEvent, FileCloseFn, FileContext, FileOpAction, FileOpEvent, FileOpFn,
    FileOpKind, FileOpenEvent, FileOpenFn, FileOpenResult, FilePattern, FilePolicyClass,
    FileReadEvent, FileReadFn, FileRwConfig, FileSession, FileSessionBuilder, FileTransformFn,
    FileWriteEvent, FileWriteFn,
};

// Re-export exec module types at top level for convenience
pub use exec::{
    ExecAction, ExecContext, ExecEvent, ExecExitFn, ExecFn, ExecPattern, ExecResult, ExecRwConfig,
    ExecSession, ExecSessionBuilder, ExecStderrFn, ExecStdinFn, ExecStdoutFn, ExecTransformFn,
    ExitEvent, StderrEvent, StdinEvent, StdoutEvent,
};

// Re-export netstat event types at top level for convenience
pub use netstat::{NetstatEvent, NetstatEventKind, NetstatEvents, NetstatFn, NetstatProtocol};

// Re-export net module types at top level for convenience
pub use net::{
    AcceptEvent, AcceptFn, AcceptResult, CloseEvent as NetCloseEvent, CloseFn as NetCloseFn,
    ConnectEvent, ConnectFn, ConnectResult, DomainEvent, DomainFn, NetAction, NetContext,
    NetDirection, NetInspectionMode, NetPattern, NetPhaseAction, NetRwConfig, NetSession,
    NetSessionBuilder, NetTransformFn, ProtocolEvent, ProtocolFn, RecvEvent, RecvFn, SendEvent,
    SendFn, TlsEvent, TlsFn,
};

// Re-export http module types at top level for convenience
pub use http::{
    HttpAction, HttpBodyEvent, HttpBodyFlags, HttpBodyMode, HttpBodySetJsonError, HttpCloseReason,
    HttpContext, HttpExchangeCloseEvent, HttpExchangeCloseFn, HttpExchangeFlags, HttpHeader,
    HttpHeaders, HttpHeadersMut, HttpMessageDoneEvent, HttpMessageKind, HttpRequestAction,
    HttpRequestBodyFn, HttpRequestDoneFn, HttpRequestEvent, HttpRequestFn, HttpRequestHead,
    HttpRequestTrailersFn, HttpResponseBodyFn, HttpResponseDoneFn, HttpResponseEvent,
    HttpResponseFn, HttpResponseHead, HttpResponseTrailersFn, HttpTrailersEvent, HttpVersion,
};

/// Prelude module - import all commonly used types.
///
/// ```rust,ignore
/// use qcontrol::prelude::*;
/// ```
pub mod prelude {
    pub use crate::buffer::Buffer;
    pub use crate::error::Error;
    pub use crate::export_plugin;
    pub use crate::logger::Logger;
    pub use crate::patterns;
    pub use crate::PluginBuilder;
    pub use crate::{parse_config, BoxError, ConfigCallback, InvalidConfig};

    // File types
    pub use crate::file::{
        FileAction, FileCloseEvent, FileCloseFn, FileContext, FileOpAction, FileOpEvent, FileOpFn,
        FileOpKind, FileOpenEvent, FileOpenFn, FileOpenResult, FilePattern, FilePolicyClass,
        FileReadEvent, FileReadFn, FileRwConfig, FileSession, FileSessionBuilder, FileTransformFn,
        FileWriteEvent, FileWriteFn,
    };
    pub use crate::{FileState, PluginState};

    // Exec types
    pub use crate::exec::{
        ExecAction, ExecContext, ExecEvent, ExecExitFn, ExecFn, ExecPattern, ExecResult,
        ExecRwConfig, ExecSession, ExecSessionBuilder, ExecStderrFn, ExecStdinFn, ExecStdoutFn,
        ExecTransformFn, ExitEvent, StderrEvent, StdinEvent, StdoutEvent,
    };

    // Net types
    pub use crate::net::{
        AcceptEvent, AcceptFn, AcceptResult, CloseEvent as NetCloseEvent, CloseFn as NetCloseFn,
        ConnectEvent, ConnectFn, ConnectResult, DomainEvent, DomainFn, NetAction, NetContext,
        NetDirection, NetInspectionMode, NetPattern, NetPhaseAction, NetRwConfig, NetSession,
        NetSessionBuilder, NetTransformFn, ProtocolEvent, ProtocolFn, RecvEvent, RecvFn, SendEvent,
        SendFn, TlsEvent, TlsFn,
    };
    pub use crate::process::{ProcessIdentity, ProcessLineageEntry};

    // Netstat event callback
    pub use crate::netstat::{
        NetstatEvent, NetstatEventKind, NetstatEvents, NetstatFn, NetstatProtocol,
    };

    // HTTP types
    pub use crate::http::{
        HttpAction, HttpBodyEvent, HttpBodyFlags, HttpBodyMode, HttpBodySetJsonError,
        HttpCloseReason, HttpContext, HttpExchangeCloseEvent, HttpExchangeCloseFn,
        HttpExchangeFlags, HttpHeader, HttpHeaders, HttpHeadersMut, HttpMessageDoneEvent,
        HttpMessageKind, HttpRequestAction, HttpRequestBodyFn, HttpRequestDoneFn, HttpRequestEvent,
        HttpRequestFn, HttpRequestHead, HttpRequestTrailersFn, HttpResponseBodyFn,
        HttpResponseDoneFn, HttpResponseEvent, HttpResponseFn, HttpResponseHead,
        HttpResponseTrailersFn, HttpTrailersEvent, HttpVersion,
    };

    // Shared stream + SSE/WebSocket types
    pub use crate::sse::{
        SseCloseEvent, SseCloseFn, SseEventFn, SseEventRecord, SseOpenEvent, SseOpenFn,
    };
    pub use crate::stream::{StreamAction, StreamContext, StreamDirection, StreamProtocol};
    pub use crate::websocket::{
        WebSocketCloseEvent, WebSocketCloseFn, WebSocketMessageEvent, WebSocketMessageFn,
        WebSocketMessageKind, WebSocketOpenEvent, WebSocketOpenFn,
    };
}
