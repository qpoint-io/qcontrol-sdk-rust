//! Network context for transform functions
//!
//! Provides metadata about the connection being operated on.

use std::ffi::CStr;

use crate::ffi;
use crate::net::NetDirection;

/// Network context passed to transform functions.
///
/// Contains all discovered information about the connection.
pub struct NetContext<'a> {
    inner: &'a ffi::qcontrol_net_ctx_t,
}

impl<'a> NetContext<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_ctx_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the connection direction.
    pub fn direction(&self) -> NetDirection {
        self.inner.direction.into()
    }

    /// Get the source address (local for outbound, remote for inbound).
    pub fn src_addr(&self) -> &str {
        if self.inner.src_addr.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.src_addr)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }

    /// Get the source port.
    pub fn src_port(&self) -> u16 {
        self.inner.src_port
    }

    /// Get the destination address (remote for outbound, local for inbound).
    pub fn dst_addr(&self) -> &str {
        if self.inner.dst_addr.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.dst_addr)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }

    /// Get the destination port.
    pub fn dst_port(&self) -> u16 {
        self.inner.dst_port
    }

    /// Check if the connection is TLS.
    pub fn is_tls(&self) -> bool {
        self.inner.is_tls != 0
    }

    /// Get the TLS version if available.
    pub fn tls_version(&self) -> Option<&str> {
        if self.inner.tls_version.is_null() || self.inner.tls_version_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.tls_version).to_str().ok() }
        }
    }

    /// Get the domain name if discovered.
    pub fn domain(&self) -> Option<&str> {
        if self.inner.domain.is_null() || self.inner.domain_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.domain).to_str().ok() }
        }
    }

    /// Get the application protocol if detected.
    pub fn protocol(&self) -> Option<&str> {
        if self.inner.protocol.is_null() || self.inner.protocol_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.protocol).to_str().ok() }
        }
    }
}
