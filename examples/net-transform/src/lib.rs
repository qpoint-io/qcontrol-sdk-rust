//! Net transform plugin - demonstrates modifying plaintext network traffic.
//!
//! This example is intended for `qcontrol wrap`.
//! It rewrites simple text responses by replacing:
//!   "hello"  -> "hullo"
//!   "server" -> "client"
//!
//! The transform is deliberately simple and best demonstrated against a local
//! text-based HTTP server such as `../test-net-transform.sh`.
//!
//! Environment variables:
//!   QCONTROL_LOG_FILE - Path to log file (default: /tmp/qcontrol.log)

use qcontrol::prelude::*;

static LOGGER: Logger = Logger::new();

fn init() -> Result<(), Error> {
    LOGGER.init();
    LOGGER.log("[net_transform.rs] initializing...");
    Ok(())
}

fn cleanup() {
    LOGGER.log("[net_transform.rs] cleanup complete");
}

fn on_net_connect(ev: &ConnectEvent) -> ConnectResult {
    if !ev.succeeded() {
        return ConnectResult::Pass;
    }

    LOGGER.log(&format!(
        "[net_transform.rs] intercepting {}:{}",
        ev.dst_addr(),
        ev.dst_port()
    ));

    ConnectResult::Session(
        NetSession::builder()
            .recv(NetRwConfig::new().patterns(vec![
                NetPattern::from_str("hello", "hullo"),
                NetPattern::from_str("server", "client"),
            ]))
            .build(),
    )
}

fn on_net_domain(_state: PluginState, ev: &DomainEvent) {
    LOGGER.log(&format!(
        "[net_transform.rs] domain(fd={}, domain={})",
        ev.fd(),
        ev.domain()
    ));
}

fn on_net_close(_state: PluginState, ev: &NetCloseEvent) {
    LOGGER.log(&format!(
        "[net_transform.rs] close(fd={}) = {}",
        ev.fd(),
        ev.result()
    ));
}

export_plugin!(PluginBuilder::new("rust_net_transform")
    .on_init(init)
    .on_cleanup(cleanup)
    .on_net_connect(on_net_connect)
    .on_net_domain(on_net_domain)
    .on_net_close(on_net_close));
