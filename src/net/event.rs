//! Network event types
//!
//! Lifetime-bound wrappers around C event structures.

use std::ffi::CStr;

use crate::ffi;

/// Event for outbound connection (connect).
pub struct ConnectEvent<'a> {
    inner: &'a ffi::qcontrol_net_connect_event_t,
}

impl<'a> ConnectEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_connect_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the destination address (IP string).
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

    /// Get the local source address (may be empty if not bound).
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

    /// Get the local source port (0 if not bound).
    pub fn src_port(&self) -> u16 {
        self.inner.src_port
    }

    /// Get the result of the connect operation.
    ///
    /// Returns 0 on success, -errno on failure.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the connect succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result == 0
    }
}

/// Event for inbound connection (accept).
pub struct AcceptEvent<'a> {
    inner: &'a ffi::qcontrol_net_accept_event_t,
}

impl<'a> AcceptEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_accept_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the accepted socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the listening socket file descriptor.
    pub fn listen_fd(&self) -> i32 {
        self.inner.listen_fd
    }

    /// Get the remote client address.
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

    /// Get the remote client port.
    pub fn src_port(&self) -> u16 {
        self.inner.src_port
    }

    /// Get the local server address.
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

    /// Get the local server port.
    pub fn dst_port(&self) -> u16 {
        self.inner.dst_port
    }

    /// Get the result of the accept operation.
    ///
    /// Returns fd on success, -errno on failure.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the accept succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result >= 0
    }
}

/// Event for TLS handshake completion.
pub struct TlsEvent<'a> {
    inner: &'a ffi::qcontrol_net_tls_event_t,
}

impl<'a> TlsEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_tls_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the TLS version (e.g., "TLSv1.2", "TLSv1.3").
    pub fn version(&self) -> &str {
        if self.inner.version.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.version)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }

    /// Get the cipher suite (may be empty).
    pub fn cipher(&self) -> Option<&str> {
        if self.inner.cipher.is_null() || self.inner.cipher_len == 0 {
            None
        } else {
            unsafe { CStr::from_ptr(self.inner.cipher).to_str().ok() }
        }
    }
}

/// Event for domain name discovery (SNI, Host header, etc.).
pub struct DomainEvent<'a> {
    inner: &'a ffi::qcontrol_net_domain_event_t,
}

impl<'a> DomainEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_domain_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the domain name.
    pub fn domain(&self) -> &str {
        if self.inner.domain.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.domain)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }
}

/// Event for application protocol detection (ALPN, content sniffing).
pub struct ProtocolEvent<'a> {
    inner: &'a ffi::qcontrol_net_protocol_event_t,
}

impl<'a> ProtocolEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_protocol_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the protocol identifier (e.g., "http/1.1", "h2").
    pub fn protocol(&self) -> &str {
        if self.inner.protocol.is_null() {
            ""
        } else {
            unsafe {
                CStr::from_ptr(self.inner.protocol)
                    .to_str()
                    .unwrap_or("<invalid utf8>")
            }
        }
    }
}

/// Event for data being sent on a connection.
pub struct SendEvent<'a> {
    inner: &'a ffi::qcontrol_net_send_event_t,
}

impl<'a> SendEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_send_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes being sent.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the data being sent.
    pub fn data(&self) -> &[u8] {
        if self.inner.buf.is_null() || self.inner.count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.inner.buf as *const u8, self.inner.count) }
        }
    }

    /// Get the data as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        std::str::from_utf8(self.data()).ok()
    }
}

/// Event for data received on a connection.
pub struct RecvEvent<'a> {
    inner: &'a ffi::qcontrol_net_recv_event_t,
}

impl<'a> RecvEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_recv_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the number of bytes requested.
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Get the result of the receive operation.
    ///
    /// On success, returns the number of bytes received (>= 0).
    /// On failure, returns a negative errno value.
    pub fn result(&self) -> isize {
        self.inner.result
    }

    /// Get the data received (only valid after successful recv).
    pub fn data(&self) -> Option<&[u8]> {
        if self.inner.result > 0 && !self.inner.buf.is_null() {
            unsafe {
                Some(std::slice::from_raw_parts(
                    self.inner.buf as *const u8,
                    self.inner.result as usize,
                ))
            }
        } else {
            None
        }
    }

    /// Get the data as a string if valid UTF-8.
    pub fn data_str(&self) -> Option<&str> {
        self.data().and_then(|d| std::str::from_utf8(d).ok())
    }
}

/// Event for connection close.
pub struct CloseEvent<'a> {
    inner: &'a ffi::qcontrol_net_close_event_t,
}

impl<'a> CloseEvent<'a> {
    /// Create from raw FFI pointer.
    ///
    /// # Safety
    /// The pointer must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut ffi::qcontrol_net_close_event_t) -> Self {
        Self { inner: &*ptr }
    }

    /// Get the socket file descriptor.
    pub fn fd(&self) -> i32 {
        self.inner.fd
    }

    /// Get the result of the close operation.
    ///
    /// Returns 0 on success, -errno on failure.
    pub fn result(&self) -> i32 {
        self.inner.result
    }

    /// Check if the close succeeded.
    pub fn succeeded(&self) -> bool {
        self.inner.result == 0
    }
}
