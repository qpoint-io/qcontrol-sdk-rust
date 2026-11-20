//! Netstat event stream for plugin callbacks.
//!
//! `on_netstat` receives batches of typed events (see
//! [`NetstatEvents`]) saying what a process's sockets did, the same on every
//! operating system. The batch borrows host memory for the callback.

mod events;
pub use events::{
    Direction, Family, Instance, InstanceEnd, NetEndpoint, NetstatEvent, NetstatEventKind,
    NetstatEvents, NetstatFn, TransportTotals, UnknownCause,
};

/// Transport protocol of a socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetstatProtocol {
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
}
