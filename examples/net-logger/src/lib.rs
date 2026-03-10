//! Net logger plugin - logs all network operations to a file
//!
//! Demonstrates the v1 network API. Note: network hooks are not yet
//! implemented in the agent, so this plugin will compile but
//! the callbacks won't be invoked at runtime.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

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
        LOGGER.log(&format!(
            "[net_logger.rs]   src={}:{}",
            src,
            ev.src_port()
        ));
    }

    ConnectResult::Pass
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
    AcceptResult::Pass
}

fn on_net_tls(_state: FileState, ev: &TlsEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] tls(fd={}, version={})",
        ev.fd(),
        ev.version()
    ));
    if let Some(cipher) = ev.cipher() {
        LOGGER.log(&format!("[net_logger.rs]   cipher={}", cipher));
    }
}

fn on_net_domain(_state: FileState, ev: &DomainEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] domain(fd={}, domain={})",
        ev.fd(),
        ev.domain()
    ));
}

fn on_net_protocol(_state: FileState, ev: &ProtocolEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] protocol(fd={}, protocol={})",
        ev.fd(),
        ev.protocol()
    ));
}

fn on_net_send(_state: FileState, ev: &SendEvent) -> NetAction {
    let preview = ev
        .data_str()
        .map(|s| s.chars().take(120).collect::<String>())
        .unwrap_or_else(|| format!("<{} bytes binary>", ev.count()));
    LOGGER.log(&format!(
        "[net_logger.rs] send(fd={}, count={}): {}",
        ev.fd(),
        ev.count(),
        preview
    ));
    NetAction::Pass
}

fn on_net_recv(_state: FileState, ev: &RecvEvent) -> NetAction {
    let preview = ev
        .data_str()
        .map(|s| s.chars().take(120).collect::<String>())
        .unwrap_or_else(|| {
            ev.data()
                .map(|d| format!("<{} bytes binary>", d.len()))
                .unwrap_or_else(|| "no data".to_string())
        });
    LOGGER.log(&format!(
        "[net_logger.rs] recv(fd={}, count={}, result={}): {}",
        ev.fd(),
        ev.count(),
        ev.result(),
        preview
    ));
    NetAction::Pass
}

fn on_net_close(_state: FileState, ev: &NetCloseEvent) {
    LOGGER.log(&format!(
        "[net_logger.rs] close(fd={}) = {}",
        ev.fd(),
        ev.result()
    ));
}

export_plugin!(
    PluginBuilder::new("rust_net_logger")
        .on_init(init)
        .on_cleanup(cleanup)
        .on_net_connect(on_net_connect)
        .on_net_accept(on_net_accept)
        .on_net_tls(on_net_tls)
        .on_net_domain(on_net_domain)
        .on_net_protocol(on_net_protocol)
        .on_net_send(on_net_send)
        .on_net_recv(on_net_recv)
        .on_net_close(on_net_close)
);
