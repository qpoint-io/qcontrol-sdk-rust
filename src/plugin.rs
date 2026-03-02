//! Plugin builder and export macro
//!
//! Provides a builder pattern for configuring plugins and the `export_plugin!`
//! macro for generating the required C ABI exports.

use crate::error::Error;
use crate::file::{FileCloseFn, FileOpenFn, FileReadFn, FileWriteFn};

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
    on_file_open: Option<FileOpenFn>,
    on_file_read: Option<FileReadFn>,
    on_file_write: Option<FileWriteFn>,
    on_file_close: Option<FileCloseFn>,
}

impl PluginBuilder {
    /// Create a new plugin builder with the given name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            on_init: None,
            on_cleanup: None,
            on_file_open: None,
            on_file_read: None,
            on_file_write: None,
            on_file_close: None,
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

        // Init wrapper
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

        // Cleanup wrapper
        extern "C" fn __qcontrol_cleanup_wrapper() {
            let builder = get_builder();
            if let Some(f) = builder.get_on_cleanup() {
                f();
            }
        }

        // File open wrapper
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

        // File read wrapper
        extern "C" fn __qcontrol_file_read_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_read_event_t,
        ) -> $crate::ffi::qcontrol_file_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_read() {
                // State is a SessionState* - extract user state from it
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

        // File write wrapper
        extern "C" fn __qcontrol_file_write_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_write_event_t,
        ) -> $crate::ffi::qcontrol_file_action_t {
            let builder = get_builder();
            if let Some(f) = builder.get_on_file_write() {
                // State is a SessionState* - extract user state from it
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

        // File close wrapper
        extern "C" fn __qcontrol_file_close_wrapper(
            state: *mut std::ffi::c_void,
            event: *mut $crate::ffi::qcontrol_file_close_event_t,
        ) {
            let builder = get_builder();

            // Call user callback if provided
            if let Some(f) = builder.get_on_file_close() {
                // State is a SessionState* - extract user state from it
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

            // Clean up SessionState if it was allocated
            if !state.is_null() {
                unsafe {
                    // The state is a Box<SessionState>
                    let _ = Box::from_raw(state as *mut $crate::file::SessionState);
                }
            }
        }

        // Static plugin descriptor wrapped for Sync
        #[no_mangle]
        #[used]
        pub static qcontrol_plugin: $crate::SyncPluginDescriptor =
            $crate::SyncPluginDescriptor($crate::ffi::qcontrol_plugin_t {
                version: $crate::ffi::QCONTROL_PLUGIN_VERSION,
                name: PLUGIN_NAME.as_ptr() as *const std::ffi::c_char,
                on_init: Some(__qcontrol_init_wrapper),
                on_cleanup: Some(__qcontrol_cleanup_wrapper),
                on_file_open: Some(__qcontrol_file_open_wrapper),
                on_file_read: Some(__qcontrol_file_read_wrapper),
                on_file_write: Some(__qcontrol_file_write_wrapper),
                on_file_close: Some(__qcontrol_file_close_wrapper),
            });
    };
}
