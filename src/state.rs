//! Generic plugin state wrapper with type-safe downcasting.
//!
//! This module owns the protocol-neutral callback state type used across file,
//! exec, net, and HTTP plugin callbacks. The wrapper exposes borrowed access
//! to one plugin-owned value without coupling that value to any specific host
//! namespace.

use std::any::Any;

/// Plugin-defined state associated with one callback lifecycle.
///
/// This wrapper provides type-safe access to the state via `downcast_ref()`,
/// mirroring the API of `std::any::Any`.
///
/// # Example
///
/// ```rust,ignore
/// struct MyState { bytes_read: usize }
///
/// fn on_read(state: PluginState, ev: &FileReadEvent) -> FileAction {
///     if let Some(s) = state.downcast_ref::<MyState>() {
///         eprintln!("Read so far: {}", s.bytes_read);
///     }
///     FileAction::Pass
/// }
/// ```
pub struct PluginState<'a> {
    inner: Option<&'a dyn Any>,
}

impl<'a> PluginState<'a> {
    /// Create a plugin state view from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a and must point to
    /// a Box<dyn Any> that was created by the SDK session/exchange wrapper.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut std::ffi::c_void) -> Self {
        if ptr.is_null() {
            Self { inner: None }
        } else {
            // The pointer is a Box<Box<dyn Any + Send>> that we cast to get the inner Any.
            let boxed = ptr as *const Box<dyn Any + Send>;
            Self {
                inner: Some(&**boxed),
            }
        }
    }

    /// Create an empty plugin state view with no associated user state.
    pub fn empty() -> Self {
        Self { inner: None }
    }

    /// Create a plugin state view from a reference to one user-owned value.
    ///
    /// Used internally when we have direct access to the user's state
    /// (for example, from a session or exchange wrapper).
    #[doc(hidden)]
    pub fn from_ref(state: &'a dyn Any) -> Self {
        Self { inner: Some(state) }
    }

    /// Check if there is state associated.
    pub fn is_some(&self) -> bool {
        self.inner.is_some()
    }

    /// Check if there is no state associated.
    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    /// Downcast to a concrete type.
    ///
    /// Returns `Some(&T)` if the state is of type `T`, `None` otherwise.
    /// This mirrors the API of `std::any::Any::downcast_ref`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// struct MyState { count: u32 }
    ///
    /// fn on_close(state: PluginState, _ev: &FileCloseEvent) {
    ///     if let Some(s) = state.downcast_ref::<MyState>() {
    ///         println!("Final count: {}", s.count);
    ///     }
    /// }
    /// ```
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.and_then(|s| s.downcast_ref::<T>())
    }
}

/// Backward-compatible alias for the original file-specific state wrapper name.
pub type FileState<'a> = PluginState<'a>;
