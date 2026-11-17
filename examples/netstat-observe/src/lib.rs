//! Example plugin that registers the netstat snapshot callback.
//!
//! The callback prints sequence, process pid, duration, and connection
//! count. Pointers are valid only for this call.

use qcontrol::prelude::*;

fn on_netstat(snapshot: &NetstatSnapshot) {
    eprintln!(
        "netstat sequence={} pid={} duration_ns={} connections={}",
        snapshot.sequence(),
        snapshot.process().pid(),
        snapshot.duration_ns(),
        snapshot.connection_count()
    );
}

export_plugin!(PluginBuilder::new("netstat-observe").on_netstat(on_netstat));
