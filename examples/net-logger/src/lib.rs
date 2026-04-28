//! Net logger plugin - logs all network operations to a file.
//!
//! This plugin is useful with `qcontrol wrap`, where wrapped HTTP and HTTPS
//! traffic is normalized into the network ABI and routed through these
//! callbacks. Native agent-side net hooks are still under development, but the
//! current implementation already exercises the same plugin-facing ABI.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();
struct TrackedState;

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[net_logger.rs] initializing...");
    Ok(())
}

fn cleanup() {
    LOGGER.log("[net_logger.rs] cleanup complete");
}

fn on_net_connect(ev: &ConnectEvent) -> ConnectResult {
    LOGGER.log(&format!(
        "[net_logger.rs] connect(fd={}, dst={}:{}) = {}",
        ev.fd(),
        ev.dst_addr(),
        ev.dst_port(),
        ev.result()
    ));

    let src = ev.src_addr();
    if !src.is_empty() {
        LOGGER.log(&format!("[net_logger.rs]   src={}:{}", src, ev.src_port()));
    }

    ConnectResult::Session(NetSession::builder().state(TrackedState).build())
}

fn on_net_accept(ev: &AcceptEvent) -> AcceptResult {
    LOGGER.log(&format!(
        "[net_logger.rs] accept(fd={}, listen_fd={}, src={}:{}) = {}",
        ev.fd(),
        ev.listen_fd(),
        ev.src_addr(),
        ev.src_port(),
        ev.result()
    ));
    AcceptResult::Session(NetSession::builder().state(TrackedState).build())
}

fn on_net_tls(_state: PluginState, ev: &TlsEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] tls(fd={}, version={})",
        ev.fd(),
        ev.version()
    ));
    if let Some(cipher) = ev.cipher() {
        LOGGER.log(&format!("[net_logger.rs]   cipher={}", cipher));
    }
}

fn on_net_domain(_state: PluginState, ev: &DomainEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] domain(fd={}, domain={})",
        ev.fd(),
        ev.domain()
    ));
}

fn on_net_protocol(_state: PluginState, ev: &ProtocolEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] protocol(fd={}, protocol={})",
        ev.fd(),
        ev.protocol()
    ));
}

fn on_net_send(_state: PluginState, ev: &SendEvent) -> NetAction {
    LOGGER.log(&format!(
        "[net_logger.rs] send(fd={}, count={})",
        ev.fd(),
        ev.count()
    ));
    NetAction::Pass
}

fn on_net_recv(_state: PluginState, ev: &RecvEvent) -> NetAction {
    LOGGER.log(&format!(
        "[net_logger.rs] recv(fd={}, count={}) = {}",
        ev.fd(),
        ev.count(),
        ev.result()
    ));
    NetAction::Pass
}

fn on_net_close(_state: PluginState, ev: &NetCloseEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] close(fd={}) = {}",
        ev.fd(),
        ev.result()
    ));
}

export_plugin!(PluginBuilder::new("rust_net_logger")
    .on_init(init)
    .on_cleanup(cleanup)
    .on_net_connect(on_net_connect)
    .on_net_accept(on_net_accept)
    .on_net_tls(on_net_tls)
    .on_net_domain(on_net_domain)
    .on_net_protocol(on_net_protocol)
    .on_net_send(on_net_send)
    .on_net_recv(on_net_recv)
    .on_net_close(on_net_close));
