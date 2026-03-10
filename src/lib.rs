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
//! - **on_init** - Called after plugin load (optional)
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
//! fn on_close(state: FileState, _ev: &FileCloseEvent) {
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
mod logger;
pub mod net;
mod plugin;

// Re-export public API
pub use buffer::Buffer;
pub use error::Error;
pub use logger::Logger;
pub use plugin::{PluginBuilder, SyncPluginDescriptor};

// Re-export file module types at top level for convenience
pub use file::{
    FileAction, FileCloseEvent, FileCloseFn, FileContext, FileOpenEvent, FileOpenFn,
    FileOpenResult, FilePattern, FileReadEvent, FileReadFn, FileRwConfig, FileSession,
    FileSessionBuilder, FileState, FileTransformFn, FileWriteEvent, FileWriteFn,
};

// Re-export exec module types at top level for convenience
pub use exec::{
    ExecAction, ExecContext, ExecEvent, ExecExitFn, ExecFn, ExecPattern, ExecResult, ExecRwConfig,
    ExecSession, ExecSessionBuilder, ExecStderrFn, ExecStdinFn, ExecStdoutFn, ExecTransformFn,
    ExitEvent, StderrEvent, StdinEvent, StdoutEvent,
};

// Re-export net module types at top level for convenience
pub use net::{
    AcceptEvent, AcceptFn, AcceptResult, CloseEvent as NetCloseEvent, CloseFn as NetCloseFn,
    ConnectEvent, ConnectFn, ConnectResult, DomainEvent, DomainFn, NetAction, NetContext,
    NetDirection, NetPattern, NetRwConfig, NetSession, NetSessionBuilder, NetTransformFn,
    ProtocolEvent, ProtocolFn, RecvEvent, RecvFn, SendEvent, SendFn, TlsEvent, TlsFn,
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

    // File types
    pub use crate::file::{
        FileAction, FileCloseEvent, FileCloseFn, FileContext, FileOpenEvent, FileOpenFn,
        FileOpenResult, FilePattern, FileReadEvent, FileReadFn, FileRwConfig, FileSession,
        FileSessionBuilder, FileState, FileTransformFn, FileWriteEvent, FileWriteFn,
    };

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
        NetDirection, NetPattern, NetRwConfig, NetSession, NetSessionBuilder, NetTransformFn,
        ProtocolEvent, ProtocolFn, RecvEvent, RecvFn, SendEvent, SendFn, TlsEvent, TlsFn,
    };
}
