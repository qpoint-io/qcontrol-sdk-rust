//! Typed netstat events for plugins that set `on_netstat`.
//!
//! Reach for it when a plugin wants to know what a process's sockets did
//! (connected, moved bytes, failed, ended) and must not care which operating
//! system the process runs on. The host hands the plugin batches of fixed-size
//! records; [`NetstatEvents`] reads one batch and yields a [`NetstatEvent`] per
//! record, every one meaning the same thing on every platform.
//!
//! A batch is valid only for the callback. Delivery timing is arbitrary, so a
//! plugin must not depend on when a batch arrives or where one ends. A
//! [`NetstatEventKind::Watermark`] at time T promises that no event for that
//! pid at or before T follows, except late transfers, and
//! [`NetstatEventKind::ProcessEnded`] is a pid's last event.
//!
//! The types mirror the records in `qcontrol/netstat.h` one for one. A record
//! whose type number this SDK does not know, or whose enum field holds a value
//! this SDK does not know, decodes to [`NetstatEventKind::Unrecognized`] rather
//! than failing, so a newer host never breaks an older plugin.

use super::NetstatProtocol;
use crate::ffi::{
    qcontrol_netstat_endpoint_t, qcontrol_netstat_event_t, QCONTROL_NETSTAT_ADDR_IPV4,
    QCONTROL_NETSTAT_ADDR_IPV6, QCONTROL_NETSTAT_DIR_RECV, QCONTROL_NETSTAT_DIR_SEND,
    QCONTROL_NETSTAT_EVENT_CONNECTED, QCONTROL_NETSTAT_EVENT_CONNECTING,
    QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED, QCONTROL_NETSTAT_EVENT_CONNECT_FAILED,
    QCONTROL_NETSTAT_EVENT_ENDED, QCONTROL_NETSTAT_EVENT_HAS_PEER, QCONTROL_NETSTAT_EVENT_LATE,
    QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT, QCONTROL_NETSTAT_EVENT_LOSSES,
    QCONTROL_NETSTAT_EVENT_NAMES_CLOSED, QCONTROL_NETSTAT_EVENT_NAME_NOT_OPEN,
    QCONTROL_NETSTAT_EVENT_OPENED, QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT,
    QCONTROL_NETSTAT_EVENT_PREEXISTING, QCONTROL_NETSTAT_EVENT_PROCESS_ARMED,
    QCONTROL_NETSTAT_EVENT_PROCESS_ENDED, QCONTROL_NETSTAT_EVENT_STOPPED,
    QCONTROL_NETSTAT_EVENT_TRANSFERRED, QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED,
    QCONTROL_NETSTAT_EVENT_UNKNOWN, QCONTROL_NETSTAT_EVENT_WATERMARK,
    QCONTROL_NETSTAT_INSTANCE_CLOSED, QCONTROL_NETSTAT_INSTANCE_REPLACED,
    QCONTROL_NETSTAT_PROTO_TCP, QCONTROL_NETSTAT_PROTO_UDP, QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST,
};

/// Callback invoked with one batch of netstat events.
///
/// The batch is valid only for this call and is never empty.
pub type NetstatFn = fn(events: &NetstatEvents<'_>);

/// One socket instance: one socket from its creation until it is gone.
///
/// The operating system reuses descriptor and handle numbers, and a program
/// can reconnect a socket to somewhere new. Each such life gets its own
/// instance number, never given to another, so two sockets' traffic is never
/// mixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instance(pub u64);

/// An address family, normalized across platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// No family, or one that is not an IP family.
    Unspecified,
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
}

impl Family {
    /// Decode a family number; any value that is not IPv4 or IPv6 is
    /// [`Family::Unspecified`].
    fn from_raw(raw: u32) -> Self {
        match raw {
            QCONTROL_NETSTAT_ADDR_IPV4 => Family::Ipv4,
            QCONTROL_NETSTAT_ADDR_IPV6 => Family::Ipv6,
            _ => Family::Unspecified,
        }
    }
}

/// An endpoint with a normalized family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetEndpoint {
    /// The address family; the address bytes are read according to it.
    pub family: Family,
    /// The port in host byte order.
    pub port: u16,
    /// The address, in network order and zero padded: four bytes for IPv4,
    /// sixteen for IPv6.
    pub addr: [u8; 16],
}

impl NetEndpoint {
    /// An endpoint that is not known: no family, port zero, and a zero
    /// address. An endpoint event carries it when an address reported earlier
    /// is no longer valid.
    pub const UNSPECIFIED: NetEndpoint = NetEndpoint {
        family: Family::Unspecified,
        port: 0,
        addr: [0; 16],
    };

    /// Decode a raw endpoint.
    fn from_raw(raw: &qcontrol_netstat_endpoint_t) -> Self {
        Self {
            family: Family::from_raw(u32::from(raw.family)),
            port: raw.port,
            addr: raw.addr,
        }
    }
}

/// The direction of a transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Bytes sent.
    Send,
    /// Bytes received.
    Recv,
}

/// Why an instance ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceEnd {
    /// The program closed it, and no send, receive, or connect still in
    /// flight refers to it.
    Closed,
    /// The program reused the socket for a new connection, which continues as
    /// a new instance.
    Replaced,
}

/// Why a stretch of a process's stream is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownCause {
    /// Reports of the process's socket activity were lost before qcontrol
    /// received them.
    RecordsLost,
}

/// Running totals of one process's reports: how many the process submitted,
/// and how many were dropped, by reason, before qcontrol received them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransportTotals {
    /// Reports the process submitted.
    pub submitted: u64,
    /// Reports dropped because the process-side buffer was full.
    pub ring_full: u64,
    /// Reports dropped because they were too large for that buffer.
    pub too_large: u64,
    /// Reports dropped because the space for report contents ran out.
    pub payload_pool_exhausted: u64,
    /// Reports dropped because they were malformed.
    pub invalid: u64,
}

/// One event about process `pid` at monotonic time `at_ns`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetstatEvent {
    /// The pid of the observed process the event is about.
    pub pid: u32,
    /// When the event happened, in monotonic nanoseconds.
    pub at_ns: u64,
    /// What happened.
    pub kind: NetstatEventKind,
}

/// What a [`NetstatEvent`] says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetstatEventKind {
    /// The program created a TCP or UDP socket. `preexisting` marks one that
    /// was already open when observation began, so its creation was not seen.
    Opened {
        instance: Instance,
        protocol: NetstatProtocol,
        family: Family,
        preexisting: bool,
    },
    /// A connection is being made to `peer` (unknown when `None`).
    Connecting {
        instance: Instance,
        peer: Option<NetEndpoint>,
    },
    /// The instance is connected to `peer` (unknown when `None`).
    Connected {
        instance: Instance,
        peer: Option<NetEndpoint>,
    },
    /// The connection attempt failed.
    ConnectFailed {
        instance: Instance,
        peer: Option<NetEndpoint>,
    },
    /// The attempt ended without a connection and without a failure the
    /// program saw. Unlike `ConnectFailed`, it does not make the instance
    /// list a row.
    ConnectAbandoned { instance: Instance },
    /// The instance's local endpoint is `endpoint`.
    LocalEndpoint {
        instance: Instance,
        endpoint: NetEndpoint,
    },
    /// The instance's peer is `endpoint`, learned after the connection event.
    PeerEndpoint {
        instance: Instance,
        endpoint: NetEndpoint,
    },
    /// `bytes` moved. `peer` is present on an unconnected UDP socket, where
    /// each datagram names its own peer. `late` marks a transfer from a record
    /// that arrived after its number was declared lost. `name_not_open` marks
    /// a call that reached the instance through a name that is no longer open.
    Transferred {
        instance: Instance,
        direction: Direction,
        bytes: u64,
        peer: Option<NetEndpoint>,
        late: bool,
        name_not_open: bool,
    },
    /// A send or receive failed; `peer` is the datagram's peer when the call
    /// named one.
    TransferFailed {
        instance: Instance,
        direction: Direction,
        peer: Option<NetEndpoint>,
    },
    /// The program closed the last of the instance's open names.
    NamesClosed { instance: Instance },
    /// The instance is gone.
    Ended {
        instance: Instance,
        reason: InstanceEnd,
    },
    /// The process's stream began.
    ProcessArmed,
    /// The process ended: all its instances are gone. A pid's last event.
    ProcessEnded,
    /// No event for this pid at or before `at_ns` will follow, except late
    /// ones. `wall_ns` is the wall-clock time of the instant `at_ns` names.
    Watermark { wall_ns: u64 },
    /// Nothing is known about the process between `from_ns` and `to_ns`.
    Unknown {
        from_ns: u64,
        to_ns: u64,
        cause: UnknownCause,
    },
    /// Running totals of the process's reports, submitted and dropped.
    Losses { transport: TransportTotals },
    /// Observation of the process stopped.
    Stopped,
    /// A record this SDK cannot decode: its type number is unknown to this
    /// SDK, or an enum field of a known type holds an unknown value. Carries
    /// the record's type number.
    Unrecognized(u16),
}

impl NetstatEvent {
    /// Decode one raw record. Never panics: a record this SDK cannot read
    /// becomes [`NetstatEventKind::Unrecognized`].
    #[must_use]
    pub fn from_raw(raw: &qcontrol_netstat_event_t) -> Self {
        Self {
            pid: raw.pid,
            at_ns: raw.at_ns,
            kind: decode_kind(raw),
        }
    }
}

/// Decode the type-specific part of `raw`.
///
/// `raw.type_` selects which member of the body union is read; each arm reads
/// only the member its type uses.
fn decode_kind(raw: &qcontrol_netstat_event_t) -> NetstatEventKind {
    let instance = Instance(raw.instance);
    let has_peer = u32::from(raw.flags) & QCONTROL_NETSTAT_EVENT_HAS_PEER != 0;
    let flag = |bit: u32| u32::from(raw.flags) & bit != 0;
    let unrecognized = NetstatEventKind::Unrecognized(raw.type_);
    // Safety (every union read below): the tag in `raw.type_` names the member
    // the host filled, and every member is plain integers valid for any
    // bit pattern, so reading it cannot produce an invalid value.
    unsafe {
        let connect_peer = || has_peer.then(|| NetEndpoint::from_raw(&raw.body.connect.peer));
        let transfer_peer = || has_peer.then(|| NetEndpoint::from_raw(&raw.body.transfer.peer));
        let direction = || match u32::from(raw.body.transfer.direction) {
            QCONTROL_NETSTAT_DIR_SEND => Some(Direction::Send),
            QCONTROL_NETSTAT_DIR_RECV => Some(Direction::Recv),
            _ => None,
        };
        match u32::from(raw.type_) {
            QCONTROL_NETSTAT_EVENT_OPENED => {
                let protocol = match u32::from(raw.body.opened.protocol) {
                    QCONTROL_NETSTAT_PROTO_TCP => NetstatProtocol::Tcp,
                    QCONTROL_NETSTAT_PROTO_UDP => NetstatProtocol::Udp,
                    _ => return unrecognized,
                };
                NetstatEventKind::Opened {
                    instance,
                    protocol,
                    family: Family::from_raw(u32::from(raw.body.opened.family)),
                    preexisting: flag(QCONTROL_NETSTAT_EVENT_PREEXISTING),
                }
            }
            QCONTROL_NETSTAT_EVENT_CONNECTING => NetstatEventKind::Connecting {
                instance,
                peer: connect_peer(),
            },
            QCONTROL_NETSTAT_EVENT_CONNECTED => NetstatEventKind::Connected {
                instance,
                peer: connect_peer(),
            },
            QCONTROL_NETSTAT_EVENT_CONNECT_FAILED => NetstatEventKind::ConnectFailed {
                instance,
                peer: connect_peer(),
            },
            QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED => {
                NetstatEventKind::ConnectAbandoned { instance }
            }
            QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT => NetstatEventKind::LocalEndpoint {
                instance,
                endpoint: NetEndpoint::from_raw(&raw.body.endpoint.endpoint),
            },
            QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT => NetstatEventKind::PeerEndpoint {
                instance,
                endpoint: NetEndpoint::from_raw(&raw.body.endpoint.endpoint),
            },
            QCONTROL_NETSTAT_EVENT_TRANSFERRED => match direction() {
                Some(direction) => NetstatEventKind::Transferred {
                    instance,
                    direction,
                    bytes: raw.body.transfer.bytes,
                    peer: transfer_peer(),
                    late: flag(QCONTROL_NETSTAT_EVENT_LATE),
                    name_not_open: flag(QCONTROL_NETSTAT_EVENT_NAME_NOT_OPEN),
                },
                None => unrecognized,
            },
            QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED => match direction() {
                Some(direction) => NetstatEventKind::TransferFailed {
                    instance,
                    direction,
                    peer: transfer_peer(),
                },
                None => unrecognized,
            },
            QCONTROL_NETSTAT_EVENT_NAMES_CLOSED => NetstatEventKind::NamesClosed { instance },
            QCONTROL_NETSTAT_EVENT_ENDED => {
                let reason = match u32::from(raw.body.ended.reason) {
                    QCONTROL_NETSTAT_INSTANCE_CLOSED => InstanceEnd::Closed,
                    QCONTROL_NETSTAT_INSTANCE_REPLACED => InstanceEnd::Replaced,
                    _ => return unrecognized,
                };
                NetstatEventKind::Ended { instance, reason }
            }
            QCONTROL_NETSTAT_EVENT_PROCESS_ARMED => NetstatEventKind::ProcessArmed,
            QCONTROL_NETSTAT_EVENT_PROCESS_ENDED => NetstatEventKind::ProcessEnded,
            QCONTROL_NETSTAT_EVENT_WATERMARK => NetstatEventKind::Watermark {
                wall_ns: raw.body.watermark.wall_ns,
            },
            QCONTROL_NETSTAT_EVENT_UNKNOWN => {
                let cause = match u32::from(raw.body.unknown.cause) {
                    QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST => UnknownCause::RecordsLost,
                    _ => return unrecognized,
                };
                NetstatEventKind::Unknown {
                    from_ns: raw.body.unknown.from_ns,
                    to_ns: raw.body.unknown.to_ns,
                    cause,
                }
            }
            QCONTROL_NETSTAT_EVENT_LOSSES => {
                let losses = &raw.body.losses;
                NetstatEventKind::Losses {
                    transport: TransportTotals {
                        submitted: losses.submitted,
                        ring_full: losses.ring_full,
                        too_large: losses.too_large,
                        payload_pool_exhausted: losses.payload_pool_exhausted,
                        invalid: losses.invalid,
                    },
                }
            }
            QCONTROL_NETSTAT_EVENT_STOPPED => NetstatEventKind::Stopped,
            _ => unrecognized,
        }
    }
}

/// One batch of netstat events, as handed to [`NetstatFn`].
///
/// Borrows host memory that is valid only for the callback.
pub struct NetstatEvents<'a> {
    raw: &'a [qcontrol_netstat_event_t],
}

impl<'a> NetstatEvents<'a> {
    /// Borrow a host-provided batch.
    ///
    /// A null pointer or a zero count is an empty batch.
    ///
    /// # Safety
    ///
    /// When `count` is non-zero, `events` must point at `count` initialized,
    /// properly aligned records that stay alive and unchanged for `'a`.
    #[must_use]
    pub unsafe fn from_raw(events: *const qcontrol_netstat_event_t, count: usize) -> Self {
        let raw = if events.is_null() || count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(events, count)
        };
        Self { raw }
    }

    /// Number of events in the batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// True when the batch holds no events.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// The events, in order: within a pid, the order they happened.
    pub fn iter(&self) -> impl Iterator<Item = NetstatEvent> + 'a {
        self.raw.iter().map(NetstatEvent::from_raw)
    }
}

impl<'a> IntoIterator for &NetstatEvents<'a> {
    type Item = NetstatEvent;
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, qcontrol_netstat_event_t>,
        fn(&qcontrol_netstat_event_t) -> NetstatEvent,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter().map(NetstatEvent::from_raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    /// The record size every language agrees on.
    const EVENT_SIZE: usize = 64;
    /// The endpoint size every language agrees on.
    const ENDPOINT_SIZE: usize = 20;
    /// Offset of the body union: the common fields (2 + 2 + 4 + 8 + 8) end here.
    const BODY_OFFSET: usize = 24;
    /// Offset of the peer inside a transfer body: `bytes` (8), `direction` (1),
    /// and `reserved` (3) come first.
    const TRANSFER_PEER_OFFSET: usize = 12;

    fn zeroed() -> qcontrol_netstat_event_t {
        unsafe { std::mem::zeroed() }
    }

    fn endpoint(family: u32, port: u16, first: u8) -> qcontrol_netstat_endpoint_t {
        let mut raw: qcontrol_netstat_endpoint_t = unsafe { std::mem::zeroed() };
        raw.family = family as u8;
        raw.port = port;
        raw.addr[0] = first;
        raw
    }

    fn record(type_: u32, instance: u64) -> qcontrol_netstat_event_t {
        let mut raw = zeroed();
        raw.type_ = type_ as u16;
        raw.pid = 4242;
        raw.at_ns = 1_000;
        raw.instance = instance;
        raw
    }

    fn decode(raw: &qcontrol_netstat_event_t) -> NetstatEventKind {
        NetstatEvent::from_raw(raw).kind
    }

    #[test]
    fn the_record_and_endpoint_have_the_agreed_layout() {
        assert_eq!(size_of::<qcontrol_netstat_event_t>(), EVENT_SIZE);
        assert_eq!(size_of::<qcontrol_netstat_endpoint_t>(), ENDPOINT_SIZE);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, type_), 0);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, flags), 2);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, pid), 4);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, at_ns), 8);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, instance), 16);
        assert_eq!(offset_of!(qcontrol_netstat_event_t, body), BODY_OFFSET);
        assert_eq!(offset_of!(qcontrol_netstat_endpoint_t, family), 0);
        assert_eq!(offset_of!(qcontrol_netstat_endpoint_t, port), 2);
        assert_eq!(offset_of!(qcontrol_netstat_endpoint_t, addr), 4);
    }

    #[test]
    fn body_members_sit_where_the_header_puts_them() {
        type Body = crate::ffi::qcontrol_netstat_event_t;
        assert_eq!(offset_of!(Body, body.opened.protocol), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.opened.family), BODY_OFFSET + 1);
        assert_eq!(offset_of!(Body, body.connect.peer), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.endpoint.endpoint), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.transfer.bytes), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.transfer.direction), BODY_OFFSET + 8);
        assert_eq!(
            offset_of!(Body, body.transfer.peer),
            BODY_OFFSET + TRANSFER_PEER_OFFSET
        );
        assert_eq!(offset_of!(Body, body.ended.reason), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.watermark.wall_ns), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.unknown.from_ns), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.unknown.to_ns), BODY_OFFSET + 8);
        assert_eq!(offset_of!(Body, body.unknown.cause), BODY_OFFSET + 16);
        assert_eq!(offset_of!(Body, body.losses.submitted), BODY_OFFSET);
        assert_eq!(offset_of!(Body, body.losses.invalid), BODY_OFFSET + 32);
    }

    #[test]
    fn opened_reads_protocol_family_and_the_preexisting_flag() {
        let mut raw = record(QCONTROL_NETSTAT_EVENT_OPENED, 7);
        raw.body.opened.protocol = QCONTROL_NETSTAT_PROTO_UDP as u8;
        raw.body.opened.family = QCONTROL_NETSTAT_ADDR_IPV6 as u8;
        raw.flags = QCONTROL_NETSTAT_EVENT_PREEXISTING as u16;
        let event = NetstatEvent::from_raw(&raw);
        assert_eq!(event.pid, 4242);
        assert_eq!(event.at_ns, 1_000);
        assert_eq!(
            event.kind,
            NetstatEventKind::Opened {
                instance: Instance(7),
                protocol: NetstatProtocol::Udp,
                family: Family::Ipv6,
                preexisting: true,
            }
        );
    }

    #[test]
    fn connect_events_carry_a_peer_only_when_the_flag_says_so() {
        for (type_, make) in [
            (
                QCONTROL_NETSTAT_EVENT_CONNECTING,
                (|instance, peer| NetstatEventKind::Connecting { instance, peer })
                    as fn(Instance, Option<NetEndpoint>) -> NetstatEventKind,
            ),
            (QCONTROL_NETSTAT_EVENT_CONNECTED, |instance, peer| {
                NetstatEventKind::Connected { instance, peer }
            }),
            (QCONTROL_NETSTAT_EVENT_CONNECT_FAILED, |instance, peer| {
                NetstatEventKind::ConnectFailed { instance, peer }
            }),
        ] {
            let mut raw = record(type_, 3);
            raw.body.connect.peer = endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 443, 10);
            assert_eq!(decode(&raw), make(Instance(3), None));
            raw.flags = QCONTROL_NETSTAT_EVENT_HAS_PEER as u16;
            let peer = NetEndpoint {
                family: Family::Ipv4,
                port: 443,
                addr: {
                    let mut addr = [0; 16];
                    addr[0] = 10;
                    addr
                },
            };
            assert_eq!(decode(&raw), make(Instance(3), Some(peer)));
        }
    }

    #[test]
    fn an_endpoint_of_each_family_decodes() {
        for (wire, family) in [
            (QCONTROL_NETSTAT_ADDR_IPV4, Family::Ipv4),
            (QCONTROL_NETSTAT_ADDR_IPV6, Family::Ipv6),
            (
                crate::ffi::QCONTROL_NETSTAT_ADDR_UNSPEC,
                Family::Unspecified,
            ),
            // A family number this SDK does not know reads as unspecified.
            (200, Family::Unspecified),
        ] {
            let mut raw = record(QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT, 1);
            raw.body.endpoint.endpoint = endpoint(wire, 8080, 9);
            let NetstatEventKind::LocalEndpoint { endpoint, .. } = decode(&raw) else {
                panic!("not a local endpoint");
            };
            assert_eq!(endpoint.family, family);
            assert_eq!(endpoint.port, 8080);
            assert_eq!(endpoint.addr[0], 9);
        }
        let mut raw = record(QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT, 1);
        raw.body.endpoint.endpoint = endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 1, 1);
        assert!(matches!(
            decode(&raw),
            NetstatEventKind::PeerEndpoint { .. }
        ));
    }

    #[test]
    fn transfers_read_direction_bytes_peer_and_flags() {
        let mut raw = record(QCONTROL_NETSTAT_EVENT_TRANSFERRED, 5);
        raw.body.transfer.bytes = 1500;
        raw.body.transfer.direction = QCONTROL_NETSTAT_DIR_RECV as u8;
        raw.body.transfer.peer = endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 53, 8);
        raw.flags = (QCONTROL_NETSTAT_EVENT_HAS_PEER
            | QCONTROL_NETSTAT_EVENT_LATE
            | QCONTROL_NETSTAT_EVENT_NAME_NOT_OPEN) as u16;
        let NetstatEventKind::Transferred {
            instance,
            direction,
            bytes,
            peer,
            late,
            name_not_open,
        } = decode(&raw)
        else {
            panic!("not a transfer");
        };
        assert_eq!(instance, Instance(5));
        assert_eq!(direction, Direction::Recv);
        assert_eq!(bytes, 1500);
        assert_eq!(peer.map(|p| p.port), Some(53));
        assert!(late && name_not_open);

        raw.flags = 0;
        raw.body.transfer.direction = QCONTROL_NETSTAT_DIR_SEND as u8;
        let NetstatEventKind::Transferred {
            direction,
            peer,
            late,
            name_not_open,
            ..
        } = decode(&raw)
        else {
            panic!("not a transfer");
        };
        assert_eq!(direction, Direction::Send);
        assert_eq!((peer, late, name_not_open), (None, false, false));
    }

    #[test]
    fn a_failed_transfer_reads_direction_and_peer() {
        let mut raw = record(QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED, 5);
        raw.body.transfer.direction = QCONTROL_NETSTAT_DIR_SEND as u8;
        assert_eq!(
            decode(&raw),
            NetstatEventKind::TransferFailed {
                instance: Instance(5),
                direction: Direction::Send,
                peer: None,
            }
        );
    }

    #[test]
    fn instance_and_process_events_decode() {
        assert_eq!(
            decode(&record(QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED, 2)),
            NetstatEventKind::ConnectAbandoned {
                instance: Instance(2)
            }
        );
        assert_eq!(
            decode(&record(QCONTROL_NETSTAT_EVENT_NAMES_CLOSED, 2)),
            NetstatEventKind::NamesClosed {
                instance: Instance(2)
            }
        );
        let mut ended = record(QCONTROL_NETSTAT_EVENT_ENDED, 2);
        ended.body.ended.reason = QCONTROL_NETSTAT_INSTANCE_REPLACED as u8;
        assert_eq!(
            decode(&ended),
            NetstatEventKind::Ended {
                instance: Instance(2),
                reason: InstanceEnd::Replaced,
            }
        );
        ended.body.ended.reason = QCONTROL_NETSTAT_INSTANCE_CLOSED as u8;
        assert!(matches!(
            decode(&ended),
            NetstatEventKind::Ended {
                reason: InstanceEnd::Closed,
                ..
            }
        ));
        assert_eq!(
            decode(&record(QCONTROL_NETSTAT_EVENT_PROCESS_ARMED, 0)),
            NetstatEventKind::ProcessArmed
        );
        assert_eq!(
            decode(&record(QCONTROL_NETSTAT_EVENT_PROCESS_ENDED, 0)),
            NetstatEventKind::ProcessEnded
        );
        assert_eq!(
            decode(&record(QCONTROL_NETSTAT_EVENT_STOPPED, 0)),
            NetstatEventKind::Stopped
        );
    }

    #[test]
    fn stream_events_decode() {
        let mut watermark = record(QCONTROL_NETSTAT_EVENT_WATERMARK, 0);
        watermark.body.watermark.wall_ns = 99;
        assert_eq!(
            decode(&watermark),
            NetstatEventKind::Watermark { wall_ns: 99 }
        );

        let mut unknown = record(QCONTROL_NETSTAT_EVENT_UNKNOWN, 0);
        unknown.body.unknown.from_ns = 10;
        unknown.body.unknown.to_ns = 20;
        unknown.body.unknown.cause = QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST as u8;
        assert_eq!(
            decode(&unknown),
            NetstatEventKind::Unknown {
                from_ns: 10,
                to_ns: 20,
                cause: UnknownCause::RecordsLost,
            }
        );

        let mut losses = record(QCONTROL_NETSTAT_EVENT_LOSSES, 0);
        losses.body.losses.submitted = 1;
        losses.body.losses.ring_full = 2;
        losses.body.losses.too_large = 3;
        losses.body.losses.payload_pool_exhausted = 4;
        losses.body.losses.invalid = 5;
        assert_eq!(
            decode(&losses),
            NetstatEventKind::Losses {
                transport: TransportTotals {
                    submitted: 1,
                    ring_full: 2,
                    too_large: 3,
                    payload_pool_exhausted: 4,
                    invalid: 5,
                },
            }
        );
    }

    #[test]
    fn an_unknown_type_number_is_unrecognized() {
        let raw = record(9999, 1);
        assert_eq!(decode(&raw), NetstatEventKind::Unrecognized(9999));
        // Zero is no type at all.
        assert_eq!(decode(&record(0, 1)), NetstatEventKind::Unrecognized(0));
    }

    #[test]
    fn an_unknown_enum_value_inside_a_known_type_is_unrecognized() {
        let mut opened = record(QCONTROL_NETSTAT_EVENT_OPENED, 1);
        opened.body.opened.protocol = 77;
        assert_eq!(
            decode(&opened),
            NetstatEventKind::Unrecognized(QCONTROL_NETSTAT_EVENT_OPENED as u16)
        );
        let mut transfer = record(QCONTROL_NETSTAT_EVENT_TRANSFERRED, 1);
        transfer.body.transfer.direction = 0;
        assert!(matches!(
            decode(&transfer),
            NetstatEventKind::Unrecognized(_)
        ));
    }

    #[test]
    fn a_batch_iterates_in_order_and_an_empty_one_is_empty() {
        let records = [
            record(QCONTROL_NETSTAT_EVENT_PROCESS_ARMED, 0),
            record(QCONTROL_NETSTAT_EVENT_STOPPED, 0),
        ];
        let batch = unsafe { NetstatEvents::from_raw(records.as_ptr(), records.len()) };
        assert_eq!(batch.len(), 2);
        let kinds: Vec<_> = batch.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            [NetstatEventKind::ProcessArmed, NetstatEventKind::Stopped]
        );
        assert_eq!((&batch).into_iter().count(), 2);

        let empty = unsafe { NetstatEvents::from_raw(std::ptr::null(), 5) };
        assert!(empty.is_empty());
        assert_eq!(empty.iter().count(), 0);
    }
}
