//! Netstat snapshot view for plugin callbacks.
//!
//! The host passes one measurement into `on_netstat`. The view borrows host
//! memory for the callback. Bytes per second is window bytes times 1e9
//! divided by [`NetstatSnapshot::duration_ns`], or zero when duration is zero.

use crate::ffi::{
    qcontrol_netstat_connection_t, qcontrol_netstat_snapshot_t, qcontrol_netstat_traffic_t,
    QCONTROL_NETSTAT_PROTO_TCP, QCONTROL_NETSTAT_PROTO_UDP,
};
use crate::process::ProcessIdentity;

/// Callback invoked with one read-only snapshot.
///
/// The snapshot is valid only for this call.
pub type NetstatFn = fn(snapshot: &NetstatSnapshot);

/// Transport protocol of a listed flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetstatProtocol {
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
}

/// Bytes and transfer failures for one interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetstatTraffic {
    /// Bytes written on sockets in this interval.
    pub bytes_sent: u64,
    /// Bytes read on sockets in this interval.
    pub bytes_recv: u64,
    /// Send calls that returned a real error in this interval.
    pub send_fail: u64,
    /// Receive calls that returned a real error in this interval.
    pub recv_fail: u64,
}

impl NetstatTraffic {
    fn from_raw(raw: qcontrol_netstat_traffic_t) -> Self {
        Self {
            bytes_sent: raw.bytes_sent,
            bytes_recv: raw.bytes_recv,
            send_fail: raw.send_fail,
            recv_fail: raw.recv_fail,
        }
    }
}

/// One listed flow inside a snapshot.
///
/// Addresses are presentation IP strings. Unknown is `None`.
pub struct NetstatConnection<'a> {
    raw: &'a qcontrol_netstat_connection_t,
}

impl<'a> NetstatConnection<'a> {
    /// TCP or UDP when the host sent a known protocol value.
    #[must_use]
    pub fn protocol(&self) -> Option<NetstatProtocol> {
        if self.raw.protocol == QCONTROL_NETSTAT_PROTO_TCP {
            Some(NetstatProtocol::Tcp)
        } else if self.raw.protocol == QCONTROL_NETSTAT_PROTO_UDP {
            Some(NetstatProtocol::Udp)
        } else {
            None
        }
    }

    /// Local IP string, or `None` if unknown.
    #[must_use]
    pub fn local_addr(&self) -> Option<&'a str> {
        cstr(self.raw.local_addr, self.raw.local_addr_len)
    }

    /// Local port in host order; 0 if unknown.
    #[must_use]
    pub fn local_port(&self) -> u16 {
        self.raw.local_port
    }

    /// Remote IP string, or `None` if unknown.
    #[must_use]
    pub fn remote_addr(&self) -> Option<&'a str> {
        cstr(self.raw.remote_addr, self.raw.remote_addr_len)
    }

    /// Remote port in host order; 0 if unknown.
    #[must_use]
    pub fn remote_port(&self) -> u16 {
        self.raw.remote_port
    }

    /// Traffic on this flow during the window.
    #[must_use]
    pub fn window(&self) -> NetstatTraffic {
        NetstatTraffic::from_raw(self.raw.window)
    }

    /// Traffic on this flow since it was recorded.
    #[must_use]
    pub fn total(&self) -> NetstatTraffic {
        NetstatTraffic::from_raw(self.raw.total)
    }
}

/// Read-only snapshot passed to [`NetstatFn`].
pub struct NetstatSnapshot {
    raw: *const qcontrol_netstat_snapshot_t,
}

impl NetstatSnapshot {
    /// Borrow a host-provided snapshot.
    ///
    /// # Safety
    ///
    /// `raw` must be non-null and point at a snapshot whose `connections`
    /// array, when `connection_count` is non-zero, stays alive for the
    /// lifetime of this value.
    #[must_use]
    pub unsafe fn from_raw(raw: *const qcontrol_netstat_snapshot_t) -> Self {
        Self { raw }
    }

    fn raw(&self) -> &qcontrol_netstat_snapshot_t {
        // The host contract is that the pointer stays live for the callback.
        unsafe { &*self.raw }
    }

    /// Sequence of this publish attempt.
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.raw().sequence
    }

    /// Process whose sockets were counted.
    #[must_use]
    pub fn process(&self) -> ProcessIdentity<'_> {
        unsafe { ProcessIdentity::from_ref(&self.raw().process) }
    }

    /// Measured length of this interval, nanoseconds.
    #[must_use]
    pub fn duration_ns(&self) -> u64 {
        self.raw().duration_ns
    }

    /// Process-wide traffic during this interval.
    #[must_use]
    pub fn window(&self) -> NetstatTraffic {
        NetstatTraffic::from_raw(self.raw().window)
    }

    /// Process-wide traffic since the publisher started.
    #[must_use]
    pub fn total(&self) -> NetstatTraffic {
        NetstatTraffic::from_raw(self.raw().total)
    }

    /// Number of listed connection rows.
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.raw().connection_count
    }

    /// Borrow connection `index` when it is inside the row array.
    #[must_use]
    pub fn connection(&self, index: usize) -> Option<NetstatConnection<'_>> {
        let raw = self.raw();
        if raw.connections.is_null() || index >= raw.connection_count {
            return None;
        }
        let row = unsafe { &*raw.connections.add(index) };
        Some(NetstatConnection { raw: row })
    }
}

fn cstr<'a>(ptr: *const std::ffi::c_char, len: usize) -> Option<&'a str> {
    if ptr.is_null() || len == 0 {
        None
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
        std::str::from_utf8(bytes).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::qcontrol_process_identity_t;
    use std::ffi::CString;
    use std::ptr;

    fn empty_identity() -> qcontrol_process_identity_t {
        qcontrol_process_identity_t {
            pid: 11,
            exec_path: ptr::null(),
            exec_path_len: 0,
            lineage: ptr::null(),
            lineage_len: 0,
        }
    }

    #[test]
    fn snapshot_exposes_duration_traffic_and_process_pid() {
        let mut raw: qcontrol_netstat_snapshot_t = unsafe { std::mem::zeroed() };
        raw.sequence = 3;
        raw.process = empty_identity();
        raw.duration_ns = 1_000_000_000;
        raw.window.bytes_sent = 100;
        raw.total.bytes_recv = 40;
        raw.connections = ptr::null();
        raw.connection_count = 0;
        let snapshot = unsafe { NetstatSnapshot::from_raw(&raw) };
        assert_eq!(snapshot.sequence(), 3);
        assert_eq!(snapshot.process().pid(), 11);
        assert!(snapshot.process().exec_path().is_none());
        assert_eq!(snapshot.duration_ns(), 1_000_000_000);
        assert_eq!(snapshot.window().bytes_sent, 100);
        assert_eq!(snapshot.total().bytes_recv, 40);
        assert_eq!(snapshot.connection_count(), 0);
        assert!(snapshot.connection(0).is_none());
    }

    #[test]
    fn connection_exposes_string_address_and_protocol_enum() {
        let local = CString::new("127.0.0.1").expect("cstr");
        let mut row: qcontrol_netstat_connection_t = unsafe { std::mem::zeroed() };
        row.protocol = QCONTROL_NETSTAT_PROTO_TCP;
        row.local_addr = local.as_ptr();
        row.local_addr_len = local.as_bytes().len();
        row.local_port = 80;
        row.window.bytes_sent = 8;
        let mut raw: qcontrol_netstat_snapshot_t = unsafe { std::mem::zeroed() };
        raw.process = empty_identity();
        raw.connections = &row;
        raw.connection_count = 1;
        let snapshot = unsafe { NetstatSnapshot::from_raw(&raw) };
        let conn = snapshot.connection(0).expect("row");
        assert_eq!(conn.protocol(), Some(NetstatProtocol::Tcp));
        assert_eq!(conn.local_addr(), Some("127.0.0.1"));
        assert_eq!(conn.local_port(), 80);
        assert!(conn.remote_addr().is_none());
        assert_eq!(conn.window().bytes_sent, 8);
    }
}
