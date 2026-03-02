//! File state wrapper with type-safe downcasting
//!
//! Provides an ergonomic wrapper for plugin-defined state that flows
//! between file operations (open -> read/write -> close).

use std::any::Any;

/// Plugin-defined state associated with a file session.
///
/// This wrapper provides type-safe access to the state via `downcast_ref()`,
/// mirroring the API of `std::any::Any`.
///
/// # Example
///
/// ```rust,ignore
/// struct MyState { bytes_read: usize }
///
/// fn on_read(state: FileState, ev: &FileReadEvent) -> FileAction {
///     if let Some(s) = state.downcast_ref::<MyState>() {
///         eprintln!("Read so far: {}", s.bytes_read);
///     }
///     FileAction::Pass
/// }
/// ```
pub struct FileState<'a> {
    inner: Option<&'a dyn Any>,
}

impl<'a> FileState<'a> {
    /// Create a FileState from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a and must point to
    /// a Box<dyn Any> that was created by FileSession.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut std::ffi::c_void) -> Self {
        if ptr.is_null() {
            Self { inner: None }
        } else {
            // The pointer is a Box<Box<dyn Any + Send>> that we cast to get the inner Any
            let boxed = ptr as *const Box<dyn Any + Send>;
            Self {
                inner: Some(&**boxed),
            }
        }
    }

    /// Create an empty FileState (no state associated).
    pub fn empty() -> Self {
        Self { inner: None }
    }

    /// Create a FileState from a reference to a dyn Any.
    ///
    /// Used internally when we have direct access to the user's state
    /// (e.g., from SessionState wrapper).
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
    /// fn on_close(state: FileState, _ev: &FileCloseEvent) {
    ///     if let Some(s) = state.downcast_ref::<MyState>() {
    ///         println!("Final count: {}", s.count);
    ///     }
    /// }
    /// ```
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.and_then(|s| s.downcast_ref::<T>())
    }
}
