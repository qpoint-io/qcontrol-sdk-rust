//! qcontrol SDK for Rust
//!
//! This crate provides idiomatic Rust bindings for the qcontrol plugin SDK,
//! enabling you to write file operation filters in safe Rust.
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
//! The new SDK uses a descriptor-based model where plugins export a single
//! `qcontrol_plugin` symbol containing all callbacks:
//!
//! - **on_init** - Called after plugin load (optional)
//! - **on_cleanup** - Called before plugin unload (optional)
//! - **on_file_open** - Called after open() syscall
//! - **on_file_read** - Called after read() syscall
//! - **on_file_write** - Called before write() syscall
//! - **on_file_close** - Called after close() syscall
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
#[doc(hidden)]
pub mod ffi;
mod error;
mod plugin;
pub mod buffer;
pub mod file;
mod logger;

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

/// Prelude module - import all commonly used types.
///
/// ```rust,ignore
/// use qcontrol::prelude::*;
/// ```
pub mod prelude {
    pub use crate::buffer::Buffer;
    pub use crate::error::Error;
    pub use crate::export_plugin;
    pub use crate::file::{
        FileAction, FileCloseEvent, FileCloseFn, FileContext, FileOpenEvent, FileOpenFn,
        FileOpenResult, FilePattern, FileReadEvent, FileReadFn, FileRwConfig, FileSession,
        FileSessionBuilder, FileState, FileTransformFn, FileWriteEvent, FileWriteFn,
    };
    pub use crate::logger::Logger;
    pub use crate::patterns;
    pub use crate::PluginBuilder;
}
