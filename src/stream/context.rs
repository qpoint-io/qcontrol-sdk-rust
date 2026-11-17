//! Shared derived-stream correlation context.
//!
//! Derived protocols such as SSE and WebSocket need to correlate back to both
//! the owning connection and the owning HTTP exchange.

use crate::ffi;
use crate::http::HttpVersion;
use crate::net::NetContext;

/// Derived stream protocol surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamProtocol {
    /// WebSocket stream created from a `101 Switching Protocols` handshake.
    WebSocket,
    /// SSE event stream carried by one `text/event-stream` HTTP response.
    Sse,
}

impl From<ffi::qcontrol_stream_protocol_t> for StreamProtocol {
    fn from(protocol: ffi::qcontrol_stream_protocol_t) -> Self {
        if protocol == ffi::qcontrol_stream_protocol_t_QCONTROL_STREAM_PROTOCOL_SSE {
            StreamProtocol::Sse
        } else {
            StreamProtocol::WebSocket
        }
    }
}

/// Direction of one derived-stream record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDirection {
    /// Record originated on the client side and flows to the server.
    ClientToServer,
    /// Record originated on the server side and flows to the client.
    ServerToClient,
    /// No directional stream record was observed.
    Unknown,
}

impl From<ffi::qcontrol_stream_direction_t> for StreamDirection {
    fn from(direction: ffi::qcontrol_stream_direction_t) -> Self {
        if direction == ffi::qcontrol_stream_direction_t_QCONTROL_STREAM_SERVER_TO_CLIENT {
            StreamDirection::ServerToClient
        } else if direction == ffi::qcontrol_stream_direction_t_QCONTROL_STREAM_DIRECTION_UNKNOWN {
            StreamDirection::Unknown
        } else {
            StreamDirection::ClientToServer
        }
    }
}

/// Shared context carried by SSE and WebSocket callbacks.
pub struct StreamContext<'a> {
    inner: &'a ffi::qcontrol_stream_ctx_t,
}

impl<'a> StreamContext<'a> {
    /// Create from a reference to the embedded FFI context.
    ///
    /// # Safety
    /// The reference must remain valid for the lifetime `'a`.
    #[doc(hidden)]
    pub unsafe fn from_ref(ctx: &'a ffi::qcontrol_stream_ctx_t) -> Self {
        Self { inner: ctx }
    }

    /// Return the normalized network context for the owning connection.
    pub fn net(&self) -> NetContext<'a> {
        unsafe { NetContext::from_ref(&self.inner.net) }
    }

    /// Return the stable identifier for the owning network connection.
    pub fn connection_id(&self) -> u64 {
        self.inner.connection_id
    }

    /// Return the stable identifier for the owning HTTP exchange.
    pub fn exchange_id(&self) -> u64 {
        self.inner.exchange_id
    }

    /// Return the HTTP version of the owning exchange.
    pub fn http_version(&self) -> HttpVersion {
        HttpVersion::from_ffi(self.inner.http_version)
    }

    /// Return which derived protocol produced this callback.
    pub fn protocol(&self) -> StreamProtocol {
        self.inner.protocol.into()
    }
}
