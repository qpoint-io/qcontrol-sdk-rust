//! File session configuration
//!
//! Session-based file plugin model where configuration happens per-file
//! at open time. State flows automatically between operations on the same fd.

use std::any::Any;
use std::ffi::{c_char, c_void, CStr};
use std::path::Path;

use crate::buffer::Buffer;
use crate::ffi;
use crate::file::{FileAction, FilePattern, FileState};

/// Transform function type for custom transforms.
///
/// Called during read/write operations to modify the buffer.
/// Receives the file state, context, and mutable buffer.
pub type FileTransformFn = fn(FileState, &FileContext, &mut Buffer) -> FileAction;

/// Internal wrapper around user state that includes transform function pointers.
///
/// This allows per-file transform functions by storing them alongside the state.
/// The agent passes this as the opaque state pointer, and our trampolines/callbacks
/// unwrap it to get both the user state and transform functions.
#[doc(hidden)]
pub struct SessionState {
    /// User-provided state (may be None if user didn't set state).
    pub user_state: Option<Box<dyn Any + Send>>,
    /// Read transform function (may be None).
    pub read_transform: Option<FileTransformFn>,
    /// Write transform function (may be None).
    pub write_transform: Option<FileTransformFn>,
    /// Owned redirected path bytes used by the exported `set_path` pointer.
    _set_path: Option<std::ffi::CString>,
}

impl SessionState {
    /// Get a FileState referencing the user's state.
    pub fn as_file_state(&self) -> FileState<'_> {
        match &self.user_state {
            Some(boxed) => FileState::from_ref(boxed.as_ref()),
            None => FileState::empty(),
        }
    }
}

/// File context passed to transform functions.
///
/// Provides metadata about the file being operated on.
pub struct FileContext<'a> {
    inner: &'a ffi::qcontrol_file_ctx_t,
}

impl<'a> FileContext<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_file_ctx_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the file path (may be empty if fd wasn't tracked from open).
    pub fn path(&self) -> &str {
        if self.inner.path.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.path as *const c_char)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }

    /// Get the file path as a Path.
    pub fn path_as_path(&self) -> &Path {
        Path::new(self.path())
    }

    /// Get the original open flags.
    pub fn flags(&self) -> i32 {
        self.inner.flags
    }
}

/// Configuration for read or write transforms.
///
/// Transform order: prefix -> replace -> transform -> suffix
#[derive(Debug, Default)]
pub struct FileRwConfig {
    /// Static prefix to prepend.
    pub prefix: Option<Vec<u8>>,
    /// Static suffix to append.
    pub suffix: Option<Vec<u8>>,
    /// Pattern replacements.
    pub patterns: Vec<FilePattern>,
    /// Custom transform function.
    pub(crate) transform: Option<FileTransformFn>,
}

impl FileRwConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a static prefix to prepend.
    pub fn prefix(mut self, prefix: impl Into<Vec<u8>>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set a static prefix string to prepend.
    pub fn prefix_str(self, prefix: &str) -> Self {
        self.prefix(prefix.as_bytes().to_vec())
    }

    /// Set a static suffix to append.
    pub fn suffix(mut self, suffix: impl Into<Vec<u8>>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Set a static suffix string to append.
    pub fn suffix_str(self, suffix: &str) -> Self {
        self.suffix(suffix.as_bytes().to_vec())
    }

    /// Add a pattern replacement.
    pub fn replace(mut self, needle: &str, replacement: &str) -> Self {
        self.patterns
            .push(FilePattern::from_str(needle, replacement));
        self
    }

    /// Add multiple pattern replacements.
    pub fn patterns(mut self, patterns: Vec<FilePattern>) -> Self {
        self.patterns.extend(patterns);
        self
    }

    /// Set a custom transform function.
    pub fn transform(mut self, f: FileTransformFn) -> Self {
        self.transform = Some(f);
        self
    }
}

/// Session configuration for a file.
///
/// Returned from `on_file_open` to configure read/write behavior
/// and associate state with the file.
pub struct FileSession {
    /// Plugin-defined state (Box<dyn Any + Send>).
    pub(crate) state: Option<Box<dyn Any + Send>>,
    /// Read transform configuration.
    pub(crate) read: Option<Box<FileRwConfig>>,
    /// Write transform configuration.
    pub(crate) write: Option<Box<FileRwConfig>>,
    /// Redirect to different path.
    pub(crate) set_path: Option<std::ffi::CString>,
    /// Override open flags (-1 = unchanged).
    pub(crate) set_flags: Option<i32>,
    /// Override file mode (0 = unchanged).
    pub(crate) set_mode: Option<u32>,
}

impl FileSession {
    /// Create a new session builder.
    pub fn builder() -> FileSessionBuilder {
        FileSessionBuilder::new()
    }

    /// Convert to FFI session structure.
    ///
    /// Note: This leaks memory intentionally - the agent is responsible
    /// for calling back to clean up via on_file_close.
    #[doc(hidden)]
    pub fn into_ffi(self) -> ffi::qcontrol_file_session_t {
        // Extract transform functions from configs before moving them
        let read_transform = self.read.as_ref().and_then(|c| c.transform);
        let write_transform = self.write.as_ref().and_then(|c| c.transform);

        // Create SessionState wrapper containing user state + transform fns
        let session_state = SessionState {
            user_state: self.state,
            read_transform,
            write_transform,
            _set_path: self.set_path,
        };
        let session_state = Box::new(session_state);

        // Leak read config
        let read_ptr = match self.read {
            Some(read) => Box::into_raw(rw_config_to_ffi(*read, true)),
            None => std::ptr::null_mut(),
        };

        // Leak write config
        let write_ptr = match self.write {
            Some(write) => Box::into_raw(rw_config_to_ffi(*write, false)),
            None => std::ptr::null_mut(),
        };

        // Keep the redirected path owned by SessionState so the exported C
        // pointer remains valid until on_file_close frees the state wrapper.
        let set_path_ptr = match session_state._set_path.as_ref() {
            Some(path) => path.as_ptr() as *const c_char,
            None => std::ptr::null(),
        };

        // Leak SessionState - will be freed in close callback
        let state_ptr = Box::into_raw(session_state) as *mut c_void;

        // Handle set_flags
        let set_flags = self.set_flags.unwrap_or(ffi::QCONTROL_FILE_FLAGS_UNCHANGED);

        // Handle set_mode
        let set_mode = self.set_mode.unwrap_or(0);

        ffi::qcontrol_file_session_t {
            state: state_ptr,
            set_path: set_path_ptr,
            set_flags,
            set_mode,
            read: read_ptr,
            write: write_ptr,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Verifies the file-session FFI conversion used by the Rust SDK.

    use super::{FileRwConfig, FileSession};

    /// Preserve the redirected path bytes behind the exported FFI pointer so
    /// file-session rewrites remain valid until close cleanup.
    #[test]
    fn into_ffi_preserves_set_path_pointer() {
        let ffi = FileSession::builder()
            .set_path("/tmp/rewritten.txt")
            .read(FileRwConfig::new().prefix_str("prefix"))
            .build()
            .into_ffi();

        let actual = unsafe { std::ffi::CStr::from_ptr(ffi.set_path) }
            .to_str()
            .expect("ffi set_path should be valid utf-8");

        assert_eq!(actual, "/tmp/rewritten.txt");

        // Match the SDK/plugin runtime cleanup model by reclaiming the leaked
        // session state and read config after the assertion completes.
        unsafe {
            let _ = Box::from_raw(ffi.state as *mut super::SessionState);
            let _ = Box::from_raw(ffi.read);
        }
    }
}

impl std::fmt::Debug for FileSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSession")
            .field("state", &self.state.is_some())
            .field("read", &self.read)
            .field("write", &self.write)
            .finish()
    }
}

/// Builder for FileSession.
#[derive(Default)]
pub struct FileSessionBuilder {
    state: Option<Box<dyn Any + Send>>,
    read: Option<Box<FileRwConfig>>,
    write: Option<Box<FileRwConfig>>,
    set_path: Option<std::ffi::CString>,
    set_flags: Option<i32>,
    set_mode: Option<u32>,
}

impl FileSessionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the plugin-defined state.
    ///
    /// The state will be passed to read/write/close callbacks.
    pub fn state<T: Any + Send + 'static>(mut self, state: T) -> Self {
        self.state = Some(Box::new(state));
        self
    }

    /// Set the read transform configuration.
    pub fn read(mut self, config: FileRwConfig) -> Self {
        self.read = Some(Box::new(config));
        self
    }

    /// Set the write transform configuration.
    pub fn write(mut self, config: FileRwConfig) -> Self {
        self.write = Some(Box::new(config));
        self
    }

    /// Redirect opens to a different path.
    ///
    /// When set, file opens will be redirected to this path instead.
    pub fn set_path(mut self, path: &str) -> Self {
        self.set_path = std::ffi::CString::new(path).ok();
        self
    }

    /// Override the open flags.
    ///
    /// When set, this value replaces the original open flags.
    pub fn set_flags(mut self, flags: i32) -> Self {
        self.set_flags = Some(flags);
        self
    }

    /// Override the file mode.
    ///
    /// When set, this value replaces the original file mode for O_CREAT.
    pub fn set_mode(mut self, mode: u32) -> Self {
        self.set_mode = Some(mode);
        self
    }

    /// Build the session.
    pub fn build(self) -> FileSession {
        FileSession {
            state: self.state,
            read: self.read,
            write: self.write,
            set_path: self.set_path,
            set_flags: self.set_flags,
            set_mode: self.set_mode,
        }
    }
}

/// Convert FileRwConfig to FFI structure.
fn rw_config_to_ffi(config: FileRwConfig, is_read: bool) -> Box<ffi::qcontrol_file_rw_config_t> {
    // Allocate patterns array if any
    let (patterns_ptr, patterns_count) = if config.patterns.is_empty() {
        (std::ptr::null(), 0)
    } else {
        let ffi_patterns: Vec<ffi::qcontrol_file_pattern_t> = config
            .patterns
            .iter()
            .map(|p| {
                // Leak the pattern data - will be cleaned up with the config
                let needle = Box::leak(p.needle().to_vec().into_boxed_slice());
                let replacement = Box::leak(p.replacement().to_vec().into_boxed_slice());
                ffi::qcontrol_file_pattern_t {
                    needle: needle.as_ptr() as *const c_char,
                    needle_len: needle.len(),
                    replacement: replacement.as_ptr() as *const c_char,
                    replacement_len: replacement.len(),
                }
            })
            .collect();
        let count = ffi_patterns.len();
        let ptr = Box::leak(ffi_patterns.into_boxed_slice()).as_ptr();
        (ptr, count)
    };

    // Handle prefix
    let (prefix_ptr, prefix_len) = match &config.prefix {
        Some(p) => {
            let leaked = Box::leak(p.clone().into_boxed_slice());
            (leaked.as_ptr() as *const c_char, leaked.len())
        }
        None => (std::ptr::null(), 0),
    };

    // Handle suffix
    let (suffix_ptr, suffix_len) = match &config.suffix {
        Some(s) => {
            let leaked = Box::leak(s.clone().into_boxed_slice());
            (leaked.as_ptr() as *const c_char, leaked.len())
        }
        None => (std::ptr::null(), 0),
    };

    // Handle transform function - use appropriate trampoline based on read/write
    let transform_fn: ffi::qcontrol_file_transform_fn = if config.transform.is_some() {
        if is_read {
            Some(read_transform_trampoline)
        } else {
            Some(write_transform_trampoline)
        }
    } else {
        None
    };

    Box::new(ffi::qcontrol_file_rw_config_t {
        prefix: prefix_ptr,
        prefix_len,
        suffix: suffix_ptr,
        suffix_len,
        prefix_fn: None,
        suffix_fn: None,
        replace: patterns_ptr,
        replace_count: patterns_count,
        transform: transform_fn,
    })
}

/// Trampoline for read transforms.
///
/// Called by the agent, extracts the transform function from SessionState,
/// and calls it with proper Rust wrappers.
unsafe extern "C" fn read_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_file_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_file_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return FileAction::Pass.to_ffi();
    }

    // Cast state to SessionState
    let session_state = &*(state as *const SessionState);

    // Get the read transform function
    let transform_fn = match session_state.read_transform {
        Some(f) => f,
        None => return FileAction::Pass.to_ffi(),
    };

    // Create Rust wrappers
    let file_state = session_state.as_file_state();
    let file_ctx = FileContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    // Call the user's transform function
    let action = transform_fn(file_state, &file_ctx, &mut buffer);
    action.to_ffi()
}

/// Trampoline for write transforms.
///
/// Called by the agent, extracts the transform function from SessionState,
/// and calls it with proper Rust wrappers.
unsafe extern "C" fn write_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_file_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_file_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return FileAction::Pass.to_ffi();
    }

    // Cast state to SessionState
    let session_state = &*(state as *const SessionState);

    // Get the write transform function
    let transform_fn = match session_state.write_transform {
        Some(f) => f,
        None => return FileAction::Pass.to_ffi(),
    };

    // Create Rust wrappers
    let file_state = session_state.as_file_state();
    let file_ctx = FileContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    // Call the user's transform function
    let action = transform_fn(file_state, &file_ctx, &mut buffer);
    action.to_ffi()
}
