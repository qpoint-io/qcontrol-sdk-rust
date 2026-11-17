//! Network context for transform functions
//!
//! Provides metadata about the connection being operated on.

use crate::ffi;
use crate::net::{str_from_ptr_len, str_opt_from_ptr_len, NetDirection};
use crate::process::ProcessIdentity;

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

    /// Create from a reference to the FFI context (embedded in another struct).
    ///
    /// # Safety
    /// The reference must be valid for the lifetime 'a.
    #[doc(hidden)]
    pub unsafe fn from_ref(ctx: &'a ffi::qcontrol_net_ctx_t) -> Self {
        Self { inner: ctx }
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
        str_from_ptr_len(self.inner.src_addr, self.inner.src_addr_len)
    }

    /// Get the source port.
    pub fn src_port(&self) -> u16 {
        self.inner.src_port
    }

    /// Get the destination address (remote for outbound, local for inbound).
    pub fn dst_addr(&self) -> &str {
        str_from_ptr_len(self.inner.dst_addr, self.inner.dst_addr_len)
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
        str_opt_from_ptr_len(self.inner.tls_version, self.inner.tls_version_len)
    }

    /// Get the TLS cipher suite if available.
    pub fn tls_cipher(&self) -> Option<&str> {
        str_opt_from_ptr_len(self.inner.tls_cipher, self.inner.tls_cipher_len)
    }

    /// Get the domain name if discovered.
    pub fn domain(&self) -> Option<&str> {
        str_opt_from_ptr_len(self.inner.domain, self.inner.domain_len)
    }

    /// Get the application protocol if detected.
    pub fn protocol(&self) -> Option<&str> {
        str_opt_from_ptr_len(self.inner.protocol, self.inner.protocol_len)
    }

    /// Get the originating process identity and executable lineage.
    pub fn process(&self) -> ProcessIdentity<'a> {
        unsafe { ProcessIdentity::from_ref(&self.inner.process) }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::NetContext;
    use crate::ffi;

    /// Build a context whose string fields are *not* NUL-terminated, mirroring
    /// the runtime's `allocator.dupe` allocation behavior.
    fn ctx_with_unterminated_fields() -> ffi::qcontrol_net_ctx_t {
        // Each backing buffer carries trailing bytes that must be ignored.
        const SRC_ADDR: &[u8] = b"127.0.0.1XX";
        const DST_ADDR: &[u8] = b"10.0.0.1XX";
        const TLS_VERSION: &[u8] = b"TLSv1.3XX";
        const TLS_CIPHER: &[u8] = b"TLS_AES_128_GCM_SHA256XX";
        const DOMAIN: &[u8] = b"example.comXX";
        const PROTOCOL: &[u8] = b"h2XX";

        ffi::qcontrol_net_ctx_t {
            fd: 5,
            direction: ffi::qcontrol_net_direction_t_QCONTROL_NET_OUTBOUND,
            src_addr: SRC_ADDR.as_ptr().cast(),
            src_addr_len: "127.0.0.1".len(),
            src_port: 4242,
            dst_addr: DST_ADDR.as_ptr().cast(),
            dst_addr_len: "10.0.0.1".len(),
            dst_port: 443,
            is_tls: 1,
            tls_version: TLS_VERSION.as_ptr().cast(),
            tls_version_len: "TLSv1.3".len(),
            tls_cipher: TLS_CIPHER.as_ptr().cast(),
            tls_cipher_len: "TLS_AES_128_GCM_SHA256".len(),
            domain: DOMAIN.as_ptr().cast(),
            domain_len: "example.com".len(),
            protocol: PROTOCOL.as_ptr().cast(),
            protocol_len: "h2".len(),
            process: ffi::qcontrol_process_identity_t {
                pid: 0,
                exec_path: ptr::null(),
                exec_path_len: 0,
                lineage: ptr::null(),
                lineage_len: 0,
            },
        }
    }

    #[test]
    fn accessors_honor_explicit_lengths_when_not_nul_terminated() {
        let inner = ctx_with_unterminated_fields();
        let ctx = unsafe { NetContext::from_ref(&inner) };

        assert_eq!(ctx.src_addr(), "127.0.0.1");
        assert_eq!(ctx.dst_addr(), "10.0.0.1");
        assert_eq!(ctx.tls_version(), Some("TLSv1.3"));
        assert_eq!(ctx.tls_cipher(), Some("TLS_AES_128_GCM_SHA256"));
        assert_eq!(ctx.domain(), Some("example.com"));
        assert_eq!(ctx.protocol(), Some("h2"));
    }

    #[test]
    fn optional_accessors_return_none_for_null_or_zero_len() {
        let inner = ffi::qcontrol_net_ctx_t {
            fd: 5,
            direction: ffi::qcontrol_net_direction_t_QCONTROL_NET_OUTBOUND,
            src_addr: ptr::null(),
            src_addr_len: 0,
            src_port: 0,
            dst_addr: ptr::null(),
            dst_addr_len: 0,
            dst_port: 0,
            is_tls: 0,
            tls_version: ptr::null(),
            tls_version_len: 0,
            tls_cipher: ptr::null(),
            tls_cipher_len: 0,
            domain: ptr::null(),
            domain_len: 0,
            protocol: ptr::null(),
            protocol_len: 0,
            process: ffi::qcontrol_process_identity_t {
                pid: 0,
                exec_path: ptr::null(),
                exec_path_len: 0,
                lineage: ptr::null(),
                lineage_len: 0,
            },
        };

        let ctx = unsafe { NetContext::from_ref(&inner) };
        assert_eq!(ctx.src_addr(), "");
        assert_eq!(ctx.dst_addr(), "");
        assert_eq!(ctx.tls_version(), None);
        assert_eq!(ctx.tls_cipher(), None);
        assert_eq!(ctx.domain(), None);
        assert_eq!(ctx.protocol(), None);
    }
}
