//! qcontrol SDK for Rust
//!
//! This crate provides idiomatic Rust bindings for the qcontrol plugin SDK,
//! enabling you to write file operation filters in safe Rust.
//!
//! # Example
//!
//! ```rust,ignore
//! use qcontrol::{plugin, register_file_open, FileOpenContext, FilterResult, Error};
//!
//! fn on_open(ctx: &FileOpenContext) -> FilterResult {
//!     eprintln!("open({}) = {}", ctx.path(), ctx.result());
//!     FilterResult::Continue
//! }
//!
//! plugin!(|| -> Result<(), Error> {
//!     register_file_open("my_plugin", None, Some(on_open))?;
//!     Ok(())
//! });
//! ```

mod ffi;
mod types;
mod file;
mod register;
mod plugin;
mod logger;

// Re-export public API from types module
pub use types::{Error, FilterHandle, FilterResult};

// Re-export public API from file module
pub use file::{FileCloseContext, FileOpenContext, FileReadContext, FileWriteContext};

// Re-export public API from register module
pub use register::{
    register_file_close, register_file_open, register_file_read, register_file_write, unregister,
    FileCloseEnterFn, FileCloseLeaveFn, FileOpenEnterFn, FileOpenLeaveFn, FileReadEnterFn,
    FileReadLeaveFn, FileWriteEnterFn, FileWriteLeaveFn,
};

// Re-export Logger
pub use logger::Logger;
