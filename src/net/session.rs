//! Network session configuration
//!
//! Session-based network plugin model where configuration happens per-connection
//! at connect/accept time. State flows automatically between I/O operations.

use std::any::Any;
use std::ffi::{c_char, c_void, CString};

use crate::buffer::Buffer;
use crate::ffi;
use crate::file::FileState;
use crate::net::{NetAction, NetContext, NetPattern};

/// Transform function type for custom transforms.
///
/// Called during send/recv operations to modify the buffer.
/// Receives the file state, context, and mutable buffer.
pub type NetTransformFn = fn(FileState, &NetContext, &mut Buffer) -> NetAction;

/// Internal wrapper around user state that includes transform function pointers.
///
/// This allows per-connection transform functions by storing them alongside the state.
#[doc(hidden)]
pub struct SessionState {
    /// User-provided state (may be None if user didn't set state).
    pub user_state: Option<Box<dyn Any + Send>>,
    /// Opaque raw state used by `ConnectResult::State` / `AcceptResult::State`.
    _opaque_state: Option<*mut c_void>,
    /// Send transform function.
    pub send_transform: Option<NetTransformFn>,
    /// Recv transform function.
    pub recv_transform: Option<NetTransformFn>,
    // Owned data for C pointers
    _set_addr: Option<CString>,
}

impl SessionState {
    /// Get a FileState referencing the user's state.
    pub fn as_file_state(&self) -> FileState<'_> {
        match &self.user_state {
            Some(boxed) => FileState::from_ref(boxed.as_ref()),
            None => FileState::empty(),
        }
    }

    /// Wrap an opaque state pointer so later callbacks and cleanup can treat it
    /// like a normal network session state container.
    #[doc(hidden)]
    pub fn from_raw_state(state: *mut c_void) -> *mut c_void {
        let session_state = SessionState {
            user_state: None,
            _opaque_state: Some(state),
            send_transform: None,
            recv_transform: None,
            _set_addr: None,
        };

        Box::into_raw(Box::new(session_state)) as *mut c_void
    }
}

/// Configuration for send/recv transforms.
///
/// Transform order: prefix -> replace -> transform -> suffix
#[derive(Debug, Default)]
pub struct NetRwConfig {
    /// Static prefix to prepend.
    pub prefix: Option<Vec<u8>>,
    /// Static suffix to append.
    pub suffix: Option<Vec<u8>>,
    /// Pattern replacements.
    pub patterns: Vec<NetPattern>,
    /// Custom transform function.
    pub(crate) transform: Option<NetTransformFn>,
}

impl NetRwConfig {
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
            .push(NetPattern::from_str(needle, replacement));
        self
    }

    /// Add multiple pattern replacements.
    pub fn patterns(mut self, patterns: Vec<NetPattern>) -> Self {
        self.patterns.extend(patterns);
        self
    }

    /// Set a custom transform function.
    pub fn transform(mut self, f: NetTransformFn) -> Self {
        self.transform = Some(f);
        self
    }
}

/// Session configuration for a network connection.
///
/// Returned from `on_net_connect`/`on_net_accept` to configure I/O behavior
/// and associate state with the connection.
pub struct NetSession {
    /// Plugin-defined state.
    pub(crate) state: Option<Box<dyn Any + Send>>,
    /// Replace destination address (connect only).
    pub(crate) set_addr: Option<CString>,
    /// Replace destination port (0 = no change, connect only).
    pub(crate) set_port: Option<u16>,
    /// Send transform configuration.
    pub(crate) send: Option<Box<NetRwConfig>>,
    /// Recv transform configuration.
    pub(crate) recv: Option<Box<NetRwConfig>>,
}

impl NetSession {
    /// Create a new session builder.
    pub fn builder() -> NetSessionBuilder {
        NetSessionBuilder::new()
    }

    /// Convert to FFI session structure.
    ///
    /// Note: This leaks memory intentionally - the agent is responsible
    /// for calling back to clean up via on_net_close.
    #[doc(hidden)]
    pub fn into_ffi(self) -> ffi::qcontrol_net_session_t {
        // Extract transform functions from configs before moving them
        let send_transform = self.send.as_ref().and_then(|c| c.transform);
        let recv_transform = self.recv.as_ref().and_then(|c| c.transform);

        // Create SessionState wrapper
        let session_state = SessionState {
            user_state: self.state,
            _opaque_state: None,
            send_transform,
            recv_transform,
            _set_addr: self.set_addr.clone(),
        };

        // Leak SessionState - will be freed in close callback
        let state_ptr = Box::into_raw(Box::new(session_state)) as *mut c_void;

        // Leak configs
        let send_ptr = match self.send {
            Some(cfg) => Box::into_raw(rw_config_to_ffi(*cfg, true)),
            None => std::ptr::null_mut(),
        };
        let recv_ptr = match self.recv {
            Some(cfg) => Box::into_raw(rw_config_to_ffi(*cfg, false)),
            None => std::ptr::null_mut(),
        };

        ffi::qcontrol_net_session_t {
            state: state_ptr,
            set_addr: self
                .set_addr
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(std::ptr::null()),
            set_port: self.set_port.unwrap_or(0),
            send_config: send_ptr,
            recv_config: recv_ptr,
        }
    }
}

impl std::fmt::Debug for NetSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetSession")
            .field("state", &self.state.is_some())
            .field("set_addr", &self.set_addr)
            .field("set_port", &self.set_port)
            .field("send", &self.send)
            .field("recv", &self.recv)
            .finish()
    }
}

/// Builder for NetSession.
#[derive(Default)]
pub struct NetSessionBuilder {
    state: Option<Box<dyn Any + Send>>,
    set_addr: Option<CString>,
    set_port: Option<u16>,
    send: Option<Box<NetRwConfig>>,
    recv: Option<Box<NetRwConfig>>,
}

impl NetSessionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the plugin-defined state.
    ///
    /// The state will be passed to send/recv/close callbacks.
    pub fn state<T: Any + Send + 'static>(mut self, state: T) -> Self {
        self.state = Some(Box::new(state));
        self
    }

    /// Replace the destination address (connect only).
    pub fn set_addr(mut self, addr: &str) -> Self {
        self.set_addr = CString::new(addr).ok();
        self
    }

    /// Replace the destination port (connect only).
    pub fn set_port(mut self, port: u16) -> Self {
        self.set_port = Some(port);
        self
    }

    /// Set the send transform configuration.
    pub fn send(mut self, config: NetRwConfig) -> Self {
        self.send = Some(Box::new(config));
        self
    }

    /// Set the recv transform configuration.
    pub fn recv(mut self, config: NetRwConfig) -> Self {
        self.recv = Some(Box::new(config));
        self
    }

    /// Build the session.
    pub fn build(self) -> NetSession {
        NetSession {
            state: self.state,
            set_addr: self.set_addr,
            set_port: self.set_port,
            send: self.send,
            recv: self.recv,
        }
    }
}

/// Convert NetRwConfig to FFI structure.
fn rw_config_to_ffi(config: NetRwConfig, is_send: bool) -> Box<ffi::qcontrol_net_rw_config_t> {
    // Allocate patterns array if any
    let (patterns_ptr, patterns_count) = if config.patterns.is_empty() {
        (std::ptr::null(), 0)
    } else {
        let ffi_patterns: Vec<ffi::qcontrol_net_pattern_t> = config
            .patterns
            .iter()
            .map(|p| {
                let needle = Box::leak(p.needle().to_vec().into_boxed_slice());
                let replacement = Box::leak(p.replacement().to_vec().into_boxed_slice());
                ffi::qcontrol_net_pattern_t {
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
    let transform_fn: ffi::qcontrol_net_transform_fn = if config.transform.is_some() {
        if is_send {
            Some(send_transform_trampoline)
        } else {
            Some(recv_transform_trampoline)
        }
    } else {
        None
    };

    Box::new(ffi::qcontrol_net_rw_config_t {
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

/// Trampoline for send transforms.
unsafe extern "C" fn send_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_net_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_net_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return NetAction::Pass.to_ffi();
    }

    let session_state = &*(state as *const SessionState);
    let transform_fn = match session_state.send_transform {
        Some(f) => f,
        None => return NetAction::Pass.to_ffi(),
    };

    let file_state = session_state.as_file_state();
    let net_ctx = NetContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    transform_fn(file_state, &net_ctx, &mut buffer).to_ffi()
}

/// Trampoline for recv transforms.
unsafe extern "C" fn recv_transform_trampoline(
    state: *mut c_void,
    ctx: *mut ffi::qcontrol_net_ctx_t,
    buf: *mut ffi::qcontrol_buffer_t,
) -> ffi::qcontrol_net_action_t {
    if state.is_null() || ctx.is_null() || buf.is_null() {
        return NetAction::Pass.to_ffi();
    }

    let session_state = &*(state as *const SessionState);
    let transform_fn = match session_state.recv_transform {
        Some(f) => f,
        None => return NetAction::Pass.to_ffi(),
    };

    let file_state = session_state.as_file_state();
    let net_ctx = NetContext::from_raw(ctx);
    let mut buffer = Buffer::from_raw(buf);

    transform_fn(file_state, &net_ctx, &mut buffer).to_ffi()
}
