//! Exec session configuration
//!
//! Session-based exec plugin model where configuration happens per-exec
//! at spawn time. State flows automatically between I/O operations.

use std::any::Any;
use std::ffi::{c_char, c_void, CString};

use crate::buffer::Buffer;
use crate::exec::{ExecAction, ExecContext, ExecPattern};
use crate::ffi;
use crate::file::FileState;

/// Transform function type for custom transforms.
///
/// Called during stdin/stdout/stderr operations to modify the buffer.
/// Receives the file state, context, and mutable buffer.
pub type ExecTransformFn = fn(FileState, &ExecContext, &mut Buffer) -> ExecAction;

/// Internal wrapper around user state that includes transform function pointers.
///
/// This allows per-exec transform functions by storing them alongside the state.
#[doc(hidden)]
pub struct SessionState {
    /// User-provided state (may be None if user didn't set state).
    pub user_state: Option<Box<dyn Any + Send>>,
    /// Stdin transform function.
    pub stdin_transform: Option<ExecTransformFn>,
    /// Stdout transform function.
    pub stdout_transform: Option<ExecTransformFn>,
    /// Stderr transform function.
    pub stderr_transform: Option<ExecTransformFn>,
    // Owned data for C pointers
    _set_path: Option<CString>,
    _argv_ptrs: Option<Vec<*const c_char>>,
    _argv_strings: Option<Vec<CString>>,
    _prepend_argv_ptrs: Option<Vec<*const c_char>>,
    _prepend_argv_strings: Option<Vec<CString>>,
    _append_argv_ptrs: Option<Vec<*const c_char>>,
    _append_argv_strings: Option<Vec<CString>>,
    _set_env_ptrs: Option<Vec<*const c_char>>,
    _set_env_strings: Option<Vec<CString>>,
    _unset_env_ptrs: Option<Vec<*const c_char>>,
    _unset_env_strings: Option<Vec<CString>>,
    _set_cwd: Option<CString>,
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

/// Configuration for stdin/stdout/stderr transforms.
///
/// Transform order: prefix -> replace -> transform -> suffix
#[derive(Debug, Default)]
pub struct ExecRwConfig {
    /// Static prefix to prepend.
    pub prefix: Option<Vec<u8>>,
    /// Static suffix to append.
    pub suffix: Option<Vec<u8>>,
    /// Pattern replacements.
    pub patterns: Vec<ExecPattern>,
    /// Custom transform function.
    pub(crate) transform: Option<ExecTransformFn>,
}

impl ExecRwConfig {
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
            .push(ExecPattern::from_str(needle, replacement));
        self
    }

    /// Add multiple pattern replacements.
    pub fn patterns(mut self, patterns: Vec<ExecPattern>) -> Self {
        self.patterns.extend(patterns);
        self
    }

    /// Set a custom transform function.
    pub fn transform(mut self, f: ExecTransformFn) -> Self {
        self.transform = Some(f);
        self
    }
}

/// Session configuration for an exec.
///
/// Returned from `on_exec` to configure I/O behavior
/// and associate state with the process.
pub struct ExecSession {
    /// Plugin-defined state.
    pub(crate) state: Option<Box<dyn Any + Send>>,
    /// Replace executable path.
    pub(crate) set_path: Option<CString>,
    /// Replace all arguments.
    pub(crate) set_argv: Option<Vec<CString>>,
    /// Arguments to prepend.
    pub(crate) prepend_argv: Option<Vec<CString>>,
    /// Arguments to append.
    pub(crate) append_argv: Option<Vec<CString>>,
    /// Environment KEY=VALUE pairs to set.
    pub(crate) set_env: Option<Vec<CString>>,
    /// Environment keys to unset.
    pub(crate) unset_env: Option<Vec<CString>>,
    /// Replace working directory.
    pub(crate) set_cwd: Option<CString>,
    /// Stdin transform configuration.
    pub(crate) stdin: Option<Box<ExecRwConfig>>,
    /// Stdout transform configuration.
    pub(crate) stdout: Option<Box<ExecRwConfig>>,
    /// Stderr transform configuration.
    pub(crate) stderr: Option<Box<ExecRwConfig>>,
}

impl ExecSession {
    /// Create a new session builder.
    pub fn builder() -> ExecSessionBuilder {
        ExecSessionBuilder::new()
    }

    /// Convert to FFI session structure.
    ///
    /// Note: This leaks memory intentionally - the agent is responsible
    /// for calling back to clean up via on_exec_exit.
    #[doc(hidden)]
    pub fn into_ffi(self) -> ffi::qcontrol_exec_session_t {
        // Extract transform functions from configs before moving them
        let stdin_transform = self.stdin.as_ref().and_then(|c| c.transform);
        let stdout_transform = self.stdout.as_ref().and_then(|c| c.transform);
        let stderr_transform = self.stderr.as_ref().and_then(|c| c.transform);

        // Convert string arrays to C format
        let (set_argv_ptrs, set_argv_strings) = strings_to_c_array(self.set_argv);
        let (prepend_argv_ptrs, prepend_argv_strings) = strings_to_c_array(self.prepend_argv);
        let (append_argv_ptrs, append_argv_strings) = strings_to_c_array(self.append_argv);
        let (set_env_ptrs, set_env_strings) = strings_to_c_array(self.set_env);
        let (unset_env_ptrs, unset_env_strings) = strings_to_c_array(self.unset_env);

        // Create SessionState wrapper
        let session_state = SessionState {
            user_state: self.state,
            stdin_transform,
            stdout_transform,
            stderr_transform,
            _set_path: self.set_path.clone(),
            _argv_ptrs: set_argv_ptrs.clone(),
            _argv_strings: set_argv_strings,
            _prepend_argv_ptrs: prepend_argv_ptrs.clone(),
            _prepend_argv_strings: prepend_argv_strings,
            _append_argv_ptrs: append_argv_ptrs.clone(),
            _append_argv_strings: append_argv_strings,
            _set_env_ptrs: set_env_ptrs.clone(),
            _set_env_strings: set_env_strings,
            _unset_env_ptrs: unset_env_ptrs.clone(),
            _unset_env_strings: unset_env_strings,
            _set_cwd: self.set_cwd.clone(),
        };

        // Leak SessionState - will be freed in exit callback
        let state_ptr = Box::into_raw(Box::new(session_state)) as *mut c_void;

        // Get pointers for C arrays
        let set_argv_ptr = set_argv_ptrs
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());
        let prepend_argv_ptr = prepend_argv_ptrs
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());
        let append_argv_ptr = append_argv_ptrs
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());
        let set_env_ptr = set_env_ptrs
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());
        let unset_env_ptr = unset_env_ptrs
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());

        // Leak configs
        let stdin_ptr = match self.stdin {
            Some(cfg) => Box::into_raw(rw_config_to_ffi(*cfg, IoType::Stdin)),
            None => std::ptr::null_mut(),
        };
        let stdout_ptr = match self.stdout {
            Some(cfg) => Box::into_raw(rw_config_to_ffi(*cfg, IoType::Stdout)),
            None => std::ptr::null_mut(),
        };
        let stderr_ptr = match self.stderr {
            Some(cfg) => Box::into_raw(rw_config_to_ffi(*cfg, IoType::Stderr)),
            None => std::ptr::null_mut(),
        };

        ffi::qcontrol_exec_session_t {
            state: state_ptr,
            set_path: self
                .set_path
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(std::ptr::null()),
            set_argv: set_argv_ptr,
            prepend_argv: prepend_argv_ptr,
            append_argv: append_argv_ptr,
            set_env: set_env_ptr,
            unset_env: unset_env_ptr,
            set_cwd: self
                .set_cwd
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(std::ptr::null()),
            stdin_config: stdin_ptr,
            stdout_config: stdout_ptr,
            stderr_config: stderr_ptr,
        }
    }
}

impl std::fmt::Debug for ExecSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecSession")
            .field("state", &self.state.is_some())
            .field("set_path", &self.set_path)
            .field("stdin", &self.stdin)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .finish()
    }
}

/// Builder for ExecSession.
#[derive(Default)]
pub struct ExecSessionBuilder {
    state: Option<Box<dyn Any + Send>>,
    set_path: Option<CString>,
    set_argv: Option<Vec<CString>>,
    prepend_argv: Option<Vec<CString>>,
    append_argv: Option<Vec<CString>>,
    set_env: Option<Vec<CString>>,
    unset_env: Option<Vec<CString>>,
    set_cwd: Option<CString>,
    stdin: Option<Box<ExecRwConfig>>,
    stdout: Option<Box<ExecRwConfig>>,
    stderr: Option<Box<ExecRwConfig>>,
}

impl ExecSessionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the plugin-defined state.
    ///
    /// The state will be passed to stdin/stdout/stderr/exit callbacks.
    pub fn state<T: Any + Send + 'static>(mut self, state: T) -> Self {
        self.state = Some(Box::new(state));
        self
    }

    /// Replace the executable path.
    pub fn set_path(mut self, path: &str) -> Self {
        self.set_path = CString::new(path).ok();
        self
    }

    /// Replace all arguments.
    pub fn set_argv(mut self, argv: &[&str]) -> Self {
        self.set_argv = Some(argv.iter().filter_map(|s| CString::new(*s).ok()).collect());
        self
    }

    /// Prepend arguments before existing.
    pub fn prepend_argv(mut self, argv: &[&str]) -> Self {
        self.prepend_argv = Some(argv.iter().filter_map(|s| CString::new(*s).ok()).collect());
        self
    }

    /// Append arguments after existing.
    pub fn append_argv(mut self, argv: &[&str]) -> Self {
        self.append_argv = Some(argv.iter().filter_map(|s| CString::new(*s).ok()).collect());
        self
    }

    /// Set environment variables (KEY=VALUE format).
    pub fn set_env(mut self, vars: &[(&str, &str)]) -> Self {
        self.set_env = Some(
            vars.iter()
                .filter_map(|(k, v)| CString::new(format!("{}={}", k, v)).ok())
                .collect(),
        );
        self
    }

    /// Unset environment variables by key.
    pub fn unset_env(mut self, keys: &[&str]) -> Self {
        self.unset_env = Some(keys.iter().filter_map(|s| CString::new(*s).ok()).collect());
        self
    }

    /// Replace the working directory.
    pub fn set_cwd(mut self, cwd: &str) -> Self {
        self.set_cwd = CString::new(cwd).ok();
        self
    }

    /// Set the stdin transform configuration.
    pub fn stdin(mut self, config: ExecRwConfig) -> Self {
        self.stdin = Some(Box::new(config));
        self
    }

    /// Set the stdout transform configuration.
    pub fn stdout(mut self, config: ExecRwConfig) -> Self {
        self.stdout = Some(Box::new(config));
        self
    }

    /// Set the stderr transform configuration.
    pub fn stderr(mut self, config: ExecRwConfig) -> Self {
        self.stderr = Some(Box::new(config));
        self
    }

    /// Build the session.
    pub fn build(self) -> ExecSession {
        ExecSession {
            state: self.state,
            set_path: self.set_path,
            set_argv: self.set_argv,
            prepend_argv: self.prepend_argv,
            append_argv: self.append_argv,
            set_env: self.set_env,
            unset_env: self.unset_env,
            set_cwd: self.set_cwd,
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: self.stderr,
        }
    }
}

/// Convert Vec<CString> to null-terminated pointer array.
fn strings_to_c_array(
    strings: Option<Vec<CString>>,
) -> (Option<Vec<*const c_char>>, Option<Vec<CString>>) {
    match strings {
        Some(strs) => {
            let mut ptrs: Vec<*const c_char> = strs.iter().map(|s| s.as_ptr()).collect();
            ptrs.push(std::ptr::null()); // NULL terminator
            (Some(ptrs), Some(strs))
        }
        None => (None, None),
    }
}

#[derive(Clone, Copy)]
enum IoType {
    Stdin,
    Stdout,
    Stderr,
}

/// Convert ExecRwConfig to FFI structure.
fn rw_config_to_ffi(config: ExecRwConfig, io_type: IoType) -> Box<ffi::qcontrol_exec_rw_config_t> {
    // Allocate patterns array if any
    let (patterns_ptr, patterns_count) = if config.patterns.is_empty() {
        (std::ptr::null(), 0)
    } else {
        let ffi_patterns: Vec<ffi::qcontrol_exec_pattern_t> = config
            .patterns
            .iter()
            .map(|p| {
                let needle = Box::leak(p.needle().to_vec().into_boxed_slice());
                let replacement = Box::leak(p.replacement().to_vec().into_boxed_slice());
                ffi::qcontrol_exec_pattern_t {
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

    // Handle transform function - use appropriate trampoline
    let transform_fn: ffi::qcontrol_exec_transform_fn = if config.transform.is_some() {
        match io_type {
            IoType::Stdin => Some(stdin_transform_trampoline),
            IoType::Stdout => Some(stdout_transform_trampoline),
            IoType::Stderr => Some(stderr_transform_trampoline),
        }
    } else {
        None
    };

    Box::new(ffi::qcontrol_exec_rw_config_t {
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

/// Trampoline for stdin transforms.
unsafe extern "C" fn stdin_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_exec_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_exec_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return ExecAction::Pass.to_ffi();
    }

    let session_state = &*(state as *const SessionState);
    let transform_fn = match session_state.stdin_transform {
        Some(f) => f,
        None => return ExecAction::Pass.to_ffi(),
    };

    let file_state = session_state.as_file_state();
    let exec_ctx = ExecContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    transform_fn(file_state, &exec_ctx, &mut buffer).to_ffi()
}

/// Trampoline for stdout transforms.
unsafe extern "C" fn stdout_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_exec_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_exec_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return ExecAction::Pass.to_ffi();
    }

    let session_state = &*(state as *const SessionState);
    let transform_fn = match session_state.stdout_transform {
        Some(f) => f,
        None => return ExecAction::Pass.to_ffi(),
    };

    let file_state = session_state.as_file_state();
    let exec_ctx = ExecContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    transform_fn(file_state, &exec_ctx, &mut buffer).to_ffi()
}

/// Trampoline for stderr transforms.
unsafe extern "C" fn stderr_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_exec_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_exec_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return ExecAction::Pass.to_ffi();
    }

    let session_state = &*(state as *const SessionState);
    let transform_fn = match session_state.stderr_transform {
        Some(f) => f,
        None => return ExecAction::Pass.to_ffi(),
    };

    let file_state = session_state.as_file_state();
    let exec_ctx = ExecContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    transform_fn(file_state, &exec_ctx, &mut buffer).to_ffi()
}
